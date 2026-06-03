//! Runtime pub/sub event bus for live room and admin updates.

use std::{collections::HashMap, sync::RwLock};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::domain::{Player, PlayerId, Room, RoomCode, RoomId, RoomStatus, SlotId};

const DEFAULT_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug)]
pub struct EventBus {
    channel_capacity: usize,
    topics: RwLock<HashMap<EventTopic, broadcast::Sender<EventMessage>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(DEFAULT_CHANNEL_CAPACITY)
    }
}

impl EventBus {
    #[must_use]
    pub fn new(channel_capacity: usize) -> Self {
        Self {
            channel_capacity,
            topics: RwLock::new(HashMap::new()),
        }
    }

    #[must_use]
    pub fn subscribe(&self, topic: EventTopic) -> broadcast::Receiver<EventMessage> {
        self.sender(topic).subscribe()
    }

    pub fn publish_admin(&self, event: RoomEvent) {
        self.publish(EventTopic::Admin, LiveEvent::Room(event));
    }

    pub fn publish_room(&self, room_id: &RoomId, event: RoomEvent) {
        self.publish(EventTopic::Room(room_id.clone()), LiveEvent::Room(event));
    }

    pub fn publish(&self, topic: EventTopic, event: LiveEvent) {
        let message = EventMessage {
            topic: topic.clone(),
            event,
        };
        let _ = self.sender(topic).send(message);
    }

    fn sender(&self, topic: EventTopic) -> broadcast::Sender<EventMessage> {
        {
            let topics = match self.topics.read() {
                Ok(topics) => topics,
                Err(poisoned) => poisoned.into_inner(),
            };
            if let Some(sender) = topics.get(&topic) {
                return sender.clone();
            }
        }

        let mut topics = match self.topics.write() {
            Ok(topics) => topics,
            Err(poisoned) => poisoned.into_inner(),
        };

        topics
            .entry(topic)
            .or_insert_with(|| {
                let (sender, _receiver) = broadcast::channel(self.channel_capacity);
                sender
            })
            .clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventTopic {
    Admin,
    Room(RoomId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMessage {
    pub topic: EventTopic,
    pub event: LiveEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveEvent {
    Room(RoomEvent),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomEvent {
    RoomCreated {
        room: RoomSummary,
    },
    PlayerJoined {
        room_id: RoomId,
        player: PublicPlayer,
    },
    PlayerLeft {
        room_id: RoomId,
        player_id: PlayerId,
    },
    PlayerRenamed {
        room_id: RoomId,
        player_id: PlayerId,
        display_name: String,
    },
    SlotClaimed {
        room_id: RoomId,
        slot_id: SlotId,
        player_id: PlayerId,
    },
    SlotReleased {
        room_id: RoomId,
        slot_id: SlotId,
    },
    SlotPromptUpdated {
        room_id: RoomId,
        slot_id: SlotId,
        updated_by: PlayerId,
    },
    RoomLocked {
        room_id: RoomId,
    },
    RoomUnlocked {
        room_id: RoomId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomSummary {
    pub id: RoomId,
    pub code: RoomCode,
    pub status: RoomStatus,
    pub version: u64,
    pub player_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicPlayer {
    pub id: PlayerId,
    pub display_name: String,
}

impl From<&Room> for RoomSummary {
    fn from(room: &Room) -> Self {
        Self {
            id: room.id.clone(),
            code: room.code.clone(),
            status: room.status,
            version: room.version,
            player_count: room.players.len(),
        }
    }
}

impl From<&Player> for PublicPlayer {
    fn from(player: &Player) -> Self {
        Self {
            id: player.id.clone(),
            display_name: player.display_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn subscriber_receives_published_room_event() {
        let bus = EventBus::default();
        let room_id = RoomId::new("room_1");
        let mut subscriber = bus.subscribe(EventTopic::Room(room_id.clone()));

        bus.publish_room(
            &room_id,
            RoomEvent::RoomLocked {
                room_id: room_id.clone(),
            },
        );

        let message = subscriber.try_recv().expect("event is available");
        assert_eq!(message.topic, EventTopic::Room(room_id.clone()));
        assert_eq!(
            message.event,
            LiveEvent::Room(RoomEvent::RoomLocked { room_id })
        );
    }

    #[tokio::test]
    async fn admin_subscriber_receives_published_admin_event() {
        let bus = EventBus::default();
        let mut subscriber = bus.subscribe(EventTopic::Admin);
        let room_id = RoomId::new("room_1");

        bus.publish_admin(RoomEvent::RoomUnlocked {
            room_id: room_id.clone(),
        });

        let message = subscriber.try_recv().expect("event is available");
        assert_eq!(message.topic, EventTopic::Admin);
        assert_eq!(
            message.event,
            LiveEvent::Room(RoomEvent::RoomUnlocked { room_id })
        );
    }

    #[tokio::test]
    async fn publish_without_subscribers_does_not_fail() {
        let bus = EventBus::default();

        bus.publish_admin(RoomEvent::RoomLocked {
            room_id: RoomId::new("room_1"),
        });
    }
}
