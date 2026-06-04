//! Room application service.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{
    domain::{
        DomainError, Player, PlayerId, Room, RoomCode, RoomId, RoomStatus, SlotId, Timestamp,
    },
    event_bus::{EventBus, RoomEvent, RoomSummary},
    persistence::{
        AddPlayerInput, ClaimSlotInput, CreateRoomInput, LockRoomInput, Persistence,
        ReleaseSlotInput, RemovePlayerInput, RenamePlayerInput, RepoError, UpdateSlotPromptInput,
    },
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinRoomOutput {
    pub room: Room,
    pub player: Player,
    pub token: String,
}

#[derive(Debug, Error)]
pub enum RoomServiceError {
    #[error("room {room_id} was not found")]
    RoomNotFound { room_id: RoomId },

    #[error(transparent)]
    Repo(#[from] RepoError),
}

pub struct RoomService<P: Persistence> {
    persistence: Arc<P>,
    event_bus: Arc<EventBus>,
}

impl<P: Persistence> RoomService<P> {
    #[must_use]
    pub fn new(persistence: Arc<P>) -> Self {
        Self {
            persistence,
            event_bus: Arc::new(EventBus::default()),
        }
    }

    #[must_use]
    pub fn with_event_bus(persistence: Arc<P>, event_bus: Arc<EventBus>) -> Self {
        Self {
            persistence,
            event_bus,
        }
    }

    pub async fn create_room(&self) -> Result<Room, RoomServiceError> {
        let sequence = next_sequence();
        let room = self
            .persistence
            .rooms()
            .create_room(CreateRoomInput {
                room_id: RoomId::new(format!("room_{sequence}")),
                code: RoomCode::new(format!("ROOM{sequence:04}")),
                timestamp: now_timestamp(),
            })
            .await?;
        self.publish_room_event(
            &room.id,
            RoomEvent::RoomCreated {
                room: RoomSummary::from(&room),
            },
        );
        Ok(room)
    }

    pub async fn join_room(
        &self,
        room_id: &RoomId,
        display_name: impl Into<String>,
    ) -> Result<JoinRoomOutput, RoomServiceError> {
        self.require_room_status(room_id, RoomStatus::Open, "join room")
            .await?;

        let sequence = next_sequence();
        let token = new_player_token(sequence);
        let (room, player) = self
            .persistence
            .rooms()
            .add_player(AddPlayerInput {
                room_id: room_id.clone(),
                player_id: PlayerId::new(format!("player_{sequence}")),
                display_name: display_name.into(),
                token_hash: hash_token(&token),
                timestamp: now_timestamp(),
            })
            .await?;

        self.publish_room_event(
            &room.id,
            RoomEvent::PlayerJoined {
                room_id: room.id.clone(),
                player: (&player).into(),
            },
        );
        Ok(JoinRoomOutput {
            room,
            player,
            token,
        })
    }

    pub async fn leave_room(
        &self,
        room_id: &RoomId,
        player_id: &PlayerId,
    ) -> Result<Room, RoomServiceError> {
        let room = self
            .persistence
            .rooms()
            .remove_player(RemovePlayerInput {
                room_id: room_id.clone(),
                player_id: player_id.clone(),
            })
            .await?;
        self.publish_room_event(
            &room.id,
            RoomEvent::PlayerLeft {
                room_id: room.id.clone(),
                player_id: player_id.clone(),
            },
        );
        Ok(room)
    }

    pub async fn rename_player(
        &self,
        room_id: &RoomId,
        player_id: &PlayerId,
        display_name: impl Into<String>,
    ) -> Result<(Room, Player), RoomServiceError> {
        self.require_room_status(room_id, RoomStatus::Open, "rename player")
            .await?;

        let result = self
            .persistence
            .rooms()
            .rename_player(RenamePlayerInput {
                room_id: room_id.clone(),
                player_id: player_id.clone(),
                display_name: display_name.into(),
            })
            .await?;
        self.publish_room_event(
            &result.0.id,
            RoomEvent::PlayerRenamed {
                room_id: result.0.id.clone(),
                player_id: player_id.clone(),
                display_name: result.1.display_name.clone(),
            },
        );
        Ok(result)
    }

    pub async fn claim_slot(
        &self,
        room_id: &RoomId,
        player_id: &PlayerId,
        slot_id: &SlotId,
    ) -> Result<Room, RoomServiceError> {
        let room = self
            .persistence
            .rooms()
            .claim_slot(ClaimSlotInput {
                room_id: room_id.clone(),
                player_id: player_id.clone(),
                slot_id: slot_id.clone(),
            })
            .await?;
        self.publish_room_event(
            &room.id,
            RoomEvent::SlotClaimed {
                room_id: room.id.clone(),
                slot_id: slot_id.clone(),
                player_id: player_id.clone(),
            },
        );
        Ok(room)
    }

    pub async fn release_slot(
        &self,
        room_id: &RoomId,
        player_id: &PlayerId,
        slot_id: &SlotId,
    ) -> Result<Room, RoomServiceError> {
        let room = self
            .persistence
            .rooms()
            .release_slot(ReleaseSlotInput {
                room_id: room_id.clone(),
                player_id: player_id.clone(),
                slot_id: slot_id.clone(),
            })
            .await?;
        self.publish_room_event(
            &room.id,
            RoomEvent::SlotReleased {
                room_id: room.id.clone(),
                slot_id: slot_id.clone(),
            },
        );
        Ok(room)
    }

    pub async fn update_prompt(
        &self,
        room_id: &RoomId,
        player_id: &PlayerId,
        slot_id: &SlotId,
        prompt: impl Into<String>,
    ) -> Result<Room, RoomServiceError> {
        let room = self
            .persistence
            .rooms()
            .update_slot_prompt(UpdateSlotPromptInput {
                room_id: room_id.clone(),
                player_id: player_id.clone(),
                slot_id: slot_id.clone(),
                prompt: prompt.into(),
            })
            .await?;
        self.publish_room_event(
            &room.id,
            RoomEvent::SlotPromptUpdated {
                room_id: room.id.clone(),
                slot_id: slot_id.clone(),
                updated_by: player_id.clone(),
            },
        );
        Ok(room)
    }

    pub async fn lock_room(&self, room_id: &RoomId) -> Result<Room, RoomServiceError> {
        let room = self
            .persistence
            .rooms()
            .lock_room(LockRoomInput {
                room_id: room_id.clone(),
            })
            .await?;
        self.publish_room_event(
            &room.id,
            RoomEvent::RoomLocked {
                room_id: room.id.clone(),
            },
        );
        Ok(room)
    }

    pub async fn unlock_room(&self, room_id: &RoomId) -> Result<Room, RoomServiceError> {
        let room = self.persistence.rooms().unlock_room(room_id).await?;
        self.publish_room_event(
            &room.id,
            RoomEvent::RoomUnlocked {
                room_id: room.id.clone(),
            },
        );
        Ok(room)
    }

    pub async fn get_room_state(&self, room_id: &RoomId) -> Result<Room, RoomServiceError> {
        self.load_room(room_id).await
    }

    pub async fn list_rooms(&self) -> Result<Vec<Room>, RoomServiceError> {
        Ok(self.persistence.rooms().list_rooms().await?)
    }

    async fn load_room(&self, room_id: &RoomId) -> Result<Room, RoomServiceError> {
        self.persistence
            .rooms()
            .get_room(room_id)
            .await?
            .ok_or_else(|| RoomServiceError::RoomNotFound {
                room_id: room_id.clone(),
            })
    }

    async fn require_room_status(
        &self,
        room_id: &RoomId,
        status: RoomStatus,
        action: &'static str,
    ) -> Result<(), RoomServiceError> {
        let room = self.load_room(room_id).await?;
        if room.status == status {
            Ok(())
        } else {
            Err(
                RepoError::Domain(DomainError::RoomStatusDoesNotAllowAction {
                    status: room.status,
                    action,
                })
                .into(),
            )
        }
    }

    fn publish_room_event(&self, room_id: &RoomId, event: RoomEvent) {
        self.event_bus.publish_room(room_id, event.clone());
        self.event_bus.publish_admin(event);
    }
}

fn next_sequence() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

fn now_timestamp() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn new_player_token(sequence: u64) -> String {
    format!("player-token-{sequence}-{}", now_timestamp())
}

fn hash_token(token: &str) -> String {
    let mut hasher = DefaultHasher::new();
    token.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{RoleName, RoomStatus},
        event_bus::{EventMessage, EventTopic, LiveEvent, RoomEvent},
        persistence::MemoryPersistence,
    };

    fn service() -> RoomService<MemoryPersistence> {
        RoomService::new(Arc::new(MemoryPersistence::new()))
    }

    fn second_player() -> PlayerId {
        PlayerId::new("missing_player")
    }

    fn assert_next_event(
        subscriber: &mut tokio::sync::broadcast::Receiver<EventMessage>,
        topic: EventTopic,
        event: RoomEvent,
    ) {
        let message = subscriber.try_recv().expect("room event is available");
        assert_eq!(message.topic, topic);
        assert_eq!(message.event, LiveEvent::Room(event));
    }

    #[tokio::test]
    async fn create_room_creates_teams_and_slots() {
        let service = service();

        let room = service.create_room().await.expect("room creation succeeds");

        assert_eq!(room.status, RoomStatus::Open);
        assert_eq!(room.teams.len(), 2);
        assert_eq!(room.slots.len(), RoleName::ALL.len() * 2);
        assert!(room.players.is_empty());
    }

    #[tokio::test]
    async fn create_room_publishes_admin_event() {
        let event_bus = Arc::new(EventBus::default());
        let service =
            RoomService::with_event_bus(Arc::new(MemoryPersistence::new()), Arc::clone(&event_bus));
        let mut admin_subscriber = event_bus.subscribe(EventTopic::Admin);

        let room = service.create_room().await.expect("room creation succeeds");

        let message = admin_subscriber
            .try_recv()
            .expect("admin event is available");
        assert_eq!(message.topic, EventTopic::Admin);
        assert_eq!(
            message.event,
            LiveEvent::Room(RoomEvent::RoomCreated {
                room: (&room).into()
            })
        );
    }

    #[tokio::test]
    async fn join_room_returns_player_and_token_without_storing_raw_token() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");

        let joined = service
            .join_room(&room.id, " Ada ")
            .await
            .expect("join succeeds");

        assert_eq!(joined.player.display_name, "Ada");
        assert!(!joined.token.is_empty());
        assert_ne!(joined.player.token_hash, joined.token);
        assert_eq!(joined.room.players[0].token_hash, joined.player.token_hash);
    }

    #[tokio::test]
    async fn join_room_publishes_room_event() {
        let event_bus = Arc::new(EventBus::default());
        let service =
            RoomService::with_event_bus(Arc::new(MemoryPersistence::new()), Arc::clone(&event_bus));
        let room = service.create_room().await.expect("room creation succeeds");
        let mut room_subscriber = event_bus.subscribe(EventTopic::Room(room.id.clone()));

        let joined = service
            .join_room(&room.id, "Ada")
            .await
            .expect("join succeeds");

        let message = room_subscriber.try_recv().expect("room event is available");
        assert_eq!(message.topic, EventTopic::Room(room.id.clone()));
        assert_eq!(
            message.event,
            LiveEvent::Room(RoomEvent::PlayerJoined {
                room_id: room.id,
                player: (&joined.player).into(),
            })
        );
    }

    #[tokio::test]
    async fn service_actions_publish_room_and_admin_events() {
        let event_bus = Arc::new(EventBus::default());
        let service =
            RoomService::with_event_bus(Arc::new(MemoryPersistence::new()), Arc::clone(&event_bus));
        let mut admin_subscriber = event_bus.subscribe(EventTopic::Admin);
        let room = service.create_room().await.expect("room creation succeeds");
        let mut room_subscriber = event_bus.subscribe(EventTopic::Room(room.id.clone()));
        let room_topic = EventTopic::Room(room.id.clone());
        let room_created = RoomEvent::RoomCreated {
            room: (&room).into(),
        };

        assert_next_event(&mut admin_subscriber, EventTopic::Admin, room_created);

        let joined = service
            .join_room(&room.id, "Ada")
            .await
            .expect("join succeeds");
        let player_joined = RoomEvent::PlayerJoined {
            room_id: room.id.clone(),
            player: (&joined.player).into(),
        };
        assert_next_event(
            &mut room_subscriber,
            room_topic.clone(),
            player_joined.clone(),
        );
        assert_next_event(&mut admin_subscriber, EventTopic::Admin, player_joined);

        let slot_id = room.slots[0].id.clone();
        service
            .claim_slot(&room.id, &joined.player.id, &slot_id)
            .await
            .expect("claim succeeds");
        let slot_claimed = RoomEvent::SlotClaimed {
            room_id: room.id.clone(),
            slot_id: slot_id.clone(),
            player_id: joined.player.id.clone(),
        };
        assert_next_event(
            &mut room_subscriber,
            room_topic.clone(),
            slot_claimed.clone(),
        );
        assert_next_event(&mut admin_subscriber, EventTopic::Admin, slot_claimed);

        service
            .update_prompt(&room.id, &joined.player.id, &slot_id, "Hold north")
            .await
            .expect("prompt update succeeds");
        let prompt_updated = RoomEvent::SlotPromptUpdated {
            room_id: room.id.clone(),
            slot_id: slot_id.clone(),
            updated_by: joined.player.id.clone(),
        };
        assert_next_event(
            &mut room_subscriber,
            room_topic.clone(),
            prompt_updated.clone(),
        );
        assert_next_event(&mut admin_subscriber, EventTopic::Admin, prompt_updated);

        service
            .rename_player(&room.id, &joined.player.id, "Grace")
            .await
            .expect("rename succeeds");
        let player_renamed = RoomEvent::PlayerRenamed {
            room_id: room.id.clone(),
            player_id: joined.player.id.clone(),
            display_name: "Grace".to_owned(),
        };
        assert_next_event(
            &mut room_subscriber,
            room_topic.clone(),
            player_renamed.clone(),
        );
        assert_next_event(&mut admin_subscriber, EventTopic::Admin, player_renamed);

        service
            .release_slot(&room.id, &joined.player.id, &slot_id)
            .await
            .expect("release succeeds");
        let slot_released = RoomEvent::SlotReleased {
            room_id: room.id.clone(),
            slot_id,
        };
        assert_next_event(
            &mut room_subscriber,
            room_topic.clone(),
            slot_released.clone(),
        );
        assert_next_event(&mut admin_subscriber, EventTopic::Admin, slot_released);

        service.lock_room(&room.id).await.expect("lock succeeds");
        let room_locked = RoomEvent::RoomLocked {
            room_id: room.id.clone(),
        };
        assert_next_event(
            &mut room_subscriber,
            room_topic.clone(),
            room_locked.clone(),
        );
        assert_next_event(&mut admin_subscriber, EventTopic::Admin, room_locked);

        service
            .unlock_room(&room.id)
            .await
            .expect("unlock succeeds");
        let room_unlocked = RoomEvent::RoomUnlocked {
            room_id: room.id.clone(),
        };
        assert_next_event(
            &mut room_subscriber,
            room_topic.clone(),
            room_unlocked.clone(),
        );
        assert_next_event(&mut admin_subscriber, EventTopic::Admin, room_unlocked);

        service
            .leave_room(&room.id, &joined.player.id)
            .await
            .expect("leave succeeds");
        let player_left = RoomEvent::PlayerLeft {
            room_id: room.id,
            player_id: joined.player.id,
        };
        assert_next_event(&mut room_subscriber, room_topic, player_left.clone());
        assert_next_event(&mut admin_subscriber, EventTopic::Admin, player_left);
    }

    #[tokio::test]
    async fn lagging_room_subscriber_does_not_fail_service_actions() {
        let event_bus = Arc::new(EventBus::new(1));
        let service =
            RoomService::with_event_bus(Arc::new(MemoryPersistence::new()), Arc::clone(&event_bus));
        let room = service.create_room().await.expect("room creation succeeds");
        let mut room_subscriber = event_bus.subscribe(EventTopic::Room(room.id.clone()));

        let joined = service
            .join_room(&room.id, "Ada")
            .await
            .expect("join succeeds");
        service
            .rename_player(&room.id, &joined.player.id, "Grace")
            .await
            .expect("rename succeeds despite lagging subscriber");

        assert!(room_subscriber.try_recv().is_err());
    }

    #[tokio::test]
    async fn claim_slot_enforces_ownership_rules() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");
        let first_slot = room.slots[0].id.clone();
        let second_slot = room.slots[1].id.clone();
        let first = service
            .join_room(&room.id, "Ada")
            .await
            .expect("first join succeeds");
        let second = service
            .join_room(&room.id, "Grace")
            .await
            .expect("second join succeeds");

        let room = service
            .claim_slot(&room.id, &first.player.id, &first_slot)
            .await
            .expect("claim succeeds");
        assert_eq!(room.slots[0].player_id.as_ref(), Some(&first.player.id));

        assert!(matches!(
            service
                .claim_slot(&room.id, &second.player.id, &first_slot)
                .await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::SlotAlreadyClaimed { .. }
            )))
        ));
        assert!(matches!(
            service
                .claim_slot(&room.id, &first.player.id, &second_slot)
                .await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::PlayerAlreadyClaimedSlot { .. }
            )))
        ));
    }

    #[tokio::test]
    async fn update_prompt_enforces_slot_ownership() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");
        let slot_id = room.slots[0].id.clone();
        let owner = service
            .join_room(&room.id, "Ada")
            .await
            .expect("owner join succeeds");
        let other = service
            .join_room(&room.id, "Grace")
            .await
            .expect("other join succeeds");

        service
            .claim_slot(&room.id, &owner.player.id, &slot_id)
            .await
            .expect("claim succeeds");

        assert!(matches!(
            service
                .update_prompt(&room.id, &other.player.id, &slot_id, "Take center")
                .await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::SlotNotOwnedByPlayer { .. }
            )))
        ));

        let room = service
            .update_prompt(&room.id, &owner.player.id, &slot_id, "Take center")
            .await
            .expect("owner prompt update succeeds");
        assert_eq!(room.slots[0].prompt_draft.as_deref(), Some("Take center"));
    }

    #[tokio::test]
    async fn release_slot_and_leave_room_clear_open_room_claims() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");
        let slot_id = room.slots[0].id.clone();
        let joined = service
            .join_room(&room.id, "Ada")
            .await
            .expect("join succeeds");

        service
            .claim_slot(&room.id, &joined.player.id, &slot_id)
            .await
            .expect("claim succeeds");
        let room = service
            .release_slot(&room.id, &joined.player.id, &slot_id)
            .await
            .expect("release succeeds");
        assert!(room.slots[0].player_id.is_none());

        service
            .claim_slot(&room.id, &joined.player.id, &slot_id)
            .await
            .expect("claim succeeds again");
        let room = service
            .leave_room(&room.id, &joined.player.id)
            .await
            .expect("leave succeeds");
        assert!(room.players.is_empty());
        assert!(room.slots[0].player_id.is_none());
    }

    #[tokio::test]
    async fn rename_player_returns_updated_player() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");
        let joined = service
            .join_room(&room.id, "Ada")
            .await
            .expect("join succeeds");

        let (room, player) = service
            .rename_player(&room.id, &joined.player.id, " Grace ")
            .await
            .expect("rename succeeds");

        assert_eq!(player.display_name, "Grace");
        assert_eq!(room.players[0].display_name, "Grace");
        assert_eq!(room.players[0].token_hash, joined.player.token_hash);
    }

    #[tokio::test]
    async fn lock_room_fills_missing_prompts_and_unlock_room_reopens() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");
        let slot_id = room.slots[0].id.clone();
        let slot_role = room.slots[0].role;
        let joined = service
            .join_room(&room.id, "Ada")
            .await
            .expect("join succeeds");

        service
            .claim_slot(&room.id, &joined.player.id, &slot_id)
            .await
            .expect("claim succeeds");
        service
            .update_prompt(&room.id, &joined.player.id, &slot_id, "Custom prompt")
            .await
            .expect("prompt update succeeds");

        let room = service.lock_room(&room.id).await.expect("lock succeeds");

        assert_eq!(room.status, RoomStatus::Locked);
        assert_eq!(
            room.slots[0].locked_prompt.as_deref(),
            Some("Custom prompt")
        );
        assert_eq!(
            room.slots[1].locked_prompt.as_deref(),
            Some(room.slots[1].role.default_prompt())
        );
        assert_eq!(room.slots[0].role, slot_role);
        assert!(room.slots.iter().all(|slot| slot.locked_prompt.is_some()));

        let room = service
            .unlock_room(&room.id)
            .await
            .expect("unlock succeeds");
        assert_eq!(room.status, RoomStatus::Open);
    }

    #[tokio::test]
    async fn get_room_state_and_list_rooms_return_current_rooms() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");

        let state = service
            .get_room_state(&room.id)
            .await
            .expect("state lookup succeeds");
        let rooms = service.list_rooms().await.expect("list succeeds");

        assert_eq!(state.id, room.id);
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0].id, room.id);
    }

    #[tokio::test]
    async fn locked_room_rejects_player_edits_and_slot_changes() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");
        let slot_id = room.slots[0].id.clone();
        let joined = service
            .join_room(&room.id, "Ada")
            .await
            .expect("join succeeds");

        service
            .claim_slot(&room.id, &joined.player.id, &slot_id)
            .await
            .expect("claim succeeds");
        service.lock_room(&room.id).await.expect("lock succeeds");

        assert!(matches!(
            service.join_room(&room.id, "Grace").await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::RoomStatusDoesNotAllowAction { .. }
            )))
        ));
        assert!(matches!(
            service
                .rename_player(&room.id, &joined.player.id, "Grace")
                .await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::RoomStatusDoesNotAllowAction { .. }
            )))
        ));
        assert!(matches!(
            service
                .claim_slot(&room.id, &joined.player.id, &room.slots[1].id)
                .await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::RoomStatusDoesNotAllowAction { .. }
            )))
        ));
        assert!(matches!(
            service
                .release_slot(&room.id, &joined.player.id, &slot_id)
                .await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::RoomStatusDoesNotAllowAction { .. }
            )))
        ));
        assert!(matches!(
            service
                .update_prompt(&room.id, &joined.player.id, &slot_id, "late edit")
                .await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::RoomStatusDoesNotAllowAction { .. }
            )))
        ));
    }

    #[tokio::test]
    async fn lifecycle_rejects_invalid_lock_unlock_and_missing_players() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");
        let slot_id = room.slots[0].id.clone();

        assert!(matches!(
            service.unlock_room(&room.id).await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::InvalidRoomStatusTransition { .. }
            )))
        ));
        assert!(matches!(
            service
                .claim_slot(&room.id, &second_player(), &slot_id)
                .await,
            Err(RoomServiceError::Repo(RepoError::PlayerNotFound { .. }))
        ));

        service.lock_room(&room.id).await.expect("lock succeeds");
        assert!(matches!(
            service.lock_room(&room.id).await,
            Err(RoomServiceError::Repo(RepoError::Domain(
                DomainError::InvalidRoomStatusTransition { .. }
            )))
        ));
    }

    #[tokio::test]
    async fn leave_after_lock_preserves_locked_prompt_snapshot() {
        let service = service();
        let room = service.create_room().await.expect("room creation succeeds");
        let slot_id = room.slots[0].id.clone();
        let joined = service
            .join_room(&room.id, "Ada")
            .await
            .expect("join succeeds");

        service
            .claim_slot(&room.id, &joined.player.id, &slot_id)
            .await
            .expect("claim succeeds");
        service
            .update_prompt(&room.id, &joined.player.id, &slot_id, "Locked snapshot")
            .await
            .expect("prompt update succeeds");
        service.lock_room(&room.id).await.expect("lock succeeds");

        let room = service
            .leave_room(&room.id, &joined.player.id)
            .await
            .expect("leave after lock succeeds");

        assert!(room.players.is_empty());
        assert_eq!(room.slots[0].player_id.as_ref(), Some(&joined.player.id));
        assert_eq!(
            room.slots[0].locked_prompt.as_deref(),
            Some("Locked snapshot")
        );
    }
}
