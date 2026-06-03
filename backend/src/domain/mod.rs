//! Domain model modules.

pub mod action;
pub mod agent;
pub mod error;
pub mod event;
pub mod ids;
pub mod map;
pub mod r#match;
pub mod player;
pub mod room;
pub mod slot;
pub mod team;

pub use action::*;
pub use agent::*;
pub use error::*;
pub use event::*;
pub use ids::*;
pub use map::*;
pub use r#match::*;
pub use player::*;
pub use room::*;
pub use slot::*;
pub use team::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_room() -> Room {
        Room::new_default(RoomId::new("room_1"), RoomCode::new("ABC123"), 100)
    }

    #[test]
    fn display_name_is_trimmed_and_allows_unicode() {
        let player = Player::new(
            PlayerId::new("player_1"),
            "  測試 Player  ",
            "token_hash",
            100,
        )
        .expect("valid display name");

        assert_eq!(player.display_name, "測試 Player");
    }

    #[test]
    fn display_name_rejects_empty_too_long_and_control_chars() {
        assert!(matches!(
            validate_display_name("   "),
            Err(DomainError::InvalidDisplayName(
                DisplayNameValidationError::Empty
            ))
        ));

        assert!(matches!(
            validate_display_name("abcdefghijklmnopqrstu"),
            Err(DomainError::InvalidDisplayName(
                DisplayNameValidationError::TooLong {
                    length: 21,
                    max: 20
                }
            ))
        ));

        assert!(matches!(
            validate_display_name("bad\nname"),
            Err(DomainError::InvalidDisplayName(
                DisplayNameValidationError::ContainsControlCharacter
            ))
        ));
    }

    #[test]
    fn default_room_has_two_teams_and_one_slot_per_role_per_team() {
        let room = test_room();

        assert_eq!(room.status, RoomStatus::Open);
        assert_eq!(room.config, RoomConfig::default());
        assert_eq!(room.teams.len(), 2);
        assert_eq!(room.slots.len(), RoleName::ALL.len() * 2);

        for team in &room.teams {
            for role in RoleName::ALL {
                assert_eq!(
                    room.slots
                        .iter()
                        .filter(|slot| slot.team_id == team.id && slot.role == role)
                        .count(),
                    1
                );
            }
        }
    }

    #[test]
    fn player_can_claim_one_slot_but_not_two() {
        let mut room = test_room();
        let player_id = PlayerId::new("player_1");
        let first_slot = room.slots[0].id.clone();
        let second_slot = room.slots[1].id.clone();

        room.claim_slot(&player_id, &first_slot)
            .expect("first claim succeeds");

        assert!(matches!(
            room.claim_slot(&player_id, &second_slot),
            Err(DomainError::PlayerAlreadyClaimedSlot { .. })
        ));
    }

    #[test]
    fn prompt_updates_require_slot_ownership() {
        let mut room = test_room();
        let owner = PlayerId::new("player_1");
        let other = PlayerId::new("player_2");
        let slot_id = room.slots[0].id.clone();

        room.claim_slot(&owner, &slot_id)
            .expect("claim should succeed");
        room.update_prompt(&owner, &slot_id, "hold north")
            .expect("owner can update prompt");

        assert_eq!(room.slots[0].prompt_draft.as_deref(), Some("hold north"));
        assert!(matches!(
            room.update_prompt(&other, &slot_id, "steal prompt"),
            Err(DomainError::SlotNotOwnedByPlayer { .. })
        ));
    }

    #[test]
    fn room_status_helpers_guard_invalid_transitions() {
        let mut room = test_room();

        assert!(matches!(
            room.mark_running(MatchId::new("match_1")),
            Err(DomainError::InvalidRoomStatusTransition {
                from: RoomStatus::Open,
                to: RoomStatus::Running,
            })
        ));

        room.lock("default prompt").expect("open room can lock");
        assert_eq!(room.status, RoomStatus::Locked);
        assert!(room.slots.iter().all(|slot| slot.locked_prompt.is_some()));

        room.mark_running(MatchId::new("match_1"))
            .expect("locked room can run");
        assert_eq!(room.status, RoomStatus::Running);
        assert!(matches!(
            room.update_prompt(
                &PlayerId::new("player_1"),
                &room.slots[0].id.clone(),
                "late"
            ),
            Err(DomainError::RoomStatusDoesNotAllowAction { .. })
        ));
    }

    #[test]
    fn default_map_has_required_shape_nodes_and_spawns() {
        let map = GameMap::default_15x9().expect("default map should validate");

        assert_eq!(map.width, 15);
        assert_eq!(map.height, 9);
        assert_eq!(map.nodes.len(), 3);
        assert!(map.has_spawn(TeamSide::A));
        assert!(map.has_spawn(TeamSide::B));
    }

    #[test]
    fn map_parser_rejects_invalid_domain_maps() {
        assert!(matches!(
            GameMap::parse(MapId::new("bad"), &["...", ".."], &[]),
            Err(DomainError::InvalidMap(MapValidationError::WrongHeight {
                actual: 2,
                expected: 9
            }))
        ));

        let rows_without_nodes = [".AAAAAAAAAAAAAB"; 9];
        assert!(matches!(
            GameMap::parse(MapId::new("bad"), &rows_without_nodes, &[]),
            Err(DomainError::InvalidMap(
                MapValidationError::WrongControlNodeCount {
                    actual: 0,
                    expected: 3
                }
            ))
        ));
    }

    #[test]
    fn role_stats_match_mvp_constants() {
        assert_eq!(
            RoleName::Vanguard.stats(),
            RoleStats {
                hp: 14,
                movement: 2,
                attack_range: 1,
                attack_damage: 2,
            }
        );
        assert_eq!(RoleName::Scout.stats().movement, 4);
        assert_eq!(RoleName::Engineer.stats().attack_range, 3);
    }
}
