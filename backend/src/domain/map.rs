//! Game map, tiles, positions, and control nodes.

use serde::{Deserialize, Serialize};

use crate::domain::{DomainError, MapId, MapValidationError, NodeId, TeamId, TeamSide, Timestamp};

pub const DEFAULT_MAP_WIDTH: usize = 15;
pub const DEFAULT_MAP_HEIGHT: usize = 9;
pub const DEFAULT_CONTROL_NODE_COUNT: usize = 3;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn manhattan_distance(self, other: Self) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Normal,
    Wall,
    Cover,
    Spawn { team_side: TeamSide },
    Node { node_id: NodeId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlNode {
    pub id: NodeId,
    pub name: String,
    pub position: Position,
    pub score_per_turn: i32,
    pub controlled_by: Option<TeamId>,
}

impl ControlNode {
    #[must_use]
    pub fn new(
        id: NodeId,
        name: impl Into<String>,
        position: Position,
        score_per_turn: i32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            position,
            score_per_turn,
            controlled_by: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMap {
    pub id: MapId,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
    pub nodes: Vec<ControlNode>,
}

impl GameMap {
    /// Parses and validates a 15x9 MVP map.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidMap`] if the map shape, tiles, control nodes, or spawns are
    /// invalid.
    pub fn parse(id: MapId, rows: &[&str], nodes: &[ControlNode]) -> Result<Self, DomainError> {
        if rows.len() != DEFAULT_MAP_HEIGHT {
            return Err(MapValidationError::WrongHeight {
                actual: rows.len(),
                expected: DEFAULT_MAP_HEIGHT,
            }
            .into());
        }

        let first_width = rows
            .first()
            .map(|row| row.chars().count())
            .unwrap_or_default();

        if first_width != DEFAULT_MAP_WIDTH {
            return Err(MapValidationError::WrongWidth {
                actual: first_width,
                expected: DEFAULT_MAP_WIDTH,
            }
            .into());
        }

        let mut tiles = Vec::with_capacity(DEFAULT_MAP_WIDTH * DEFAULT_MAP_HEIGHT);
        let mut node_count = 0;
        let mut has_spawn_a = false;
        let mut has_spawn_b = false;

        for (y, row) in rows.iter().enumerate() {
            let row_width = row.chars().count();
            if row_width != first_width {
                return Err(MapValidationError::UnequalRowWidth {
                    row: y,
                    actual: row_width,
                    expected: first_width,
                }
                .into());
            }

            for (x, tile_char) in row.chars().enumerate() {
                let position = Position::new(x as i32, y as i32);
                let tile = match tile_char {
                    '.' => Tile::Normal,
                    '#' => Tile::Wall,
                    'o' => Tile::Cover,
                    'A' => {
                        has_spawn_a = true;
                        Tile::Spawn {
                            team_side: TeamSide::A,
                        }
                    }
                    'B' => {
                        has_spawn_b = true;
                        Tile::Spawn {
                            team_side: TeamSide::B,
                        }
                    }
                    'N' | 'C' | 'S' => {
                        node_count += 1;
                        let node = nodes.iter().find(|node| node.position == position).ok_or(
                            MapValidationError::WrongControlNodeCount {
                                actual: nodes.len(),
                                expected: DEFAULT_CONTROL_NODE_COUNT,
                            },
                        )?;
                        Tile::Node {
                            node_id: node.id.clone(),
                        }
                    }
                    other => {
                        return Err(MapValidationError::InvalidTile {
                            tile: other,
                            x: x as i32,
                            y: y as i32,
                        }
                        .into());
                    }
                };

                tiles.push(tile);
            }
        }

        if node_count != DEFAULT_CONTROL_NODE_COUNT || nodes.len() != DEFAULT_CONTROL_NODE_COUNT {
            return Err(MapValidationError::WrongControlNodeCount {
                actual: node_count,
                expected: DEFAULT_CONTROL_NODE_COUNT,
            }
            .into());
        }

        if !has_spawn_a {
            return Err(MapValidationError::MissingSpawn { side: TeamSide::A }.into());
        }

        if !has_spawn_b {
            return Err(MapValidationError::MissingSpawn { side: TeamSide::B }.into());
        }

        Ok(Self {
            id,
            width: DEFAULT_MAP_WIDTH,
            height: DEFAULT_MAP_HEIGHT,
            tiles,
            nodes: nodes.to_vec(),
        })
    }

    /// Builds the default 15x9 MVP map.
    ///
    /// # Errors
    ///
    /// Returns a domain error if the embedded map definition is invalid.
    pub fn default_15x9() -> Result<Self, DomainError> {
        Self::parse(
            MapId::new("default-15x9"),
            &[
                "...#..oNo..#...",
                "AA.#.......#.BB",
                "AA...#o.o#...BB",
                "...o.......o...",
                ".A...o.C.o...B.",
                "...o.......o...",
                "AA...#o.o#...BB",
                "AA.#.......#.BB",
                "...#..oSo..#...",
            ],
            &default_control_nodes(),
        )
    }

    #[must_use]
    pub fn has_spawn(&self, side: TeamSide) -> bool {
        self.tiles
            .iter()
            .any(|tile| matches!(tile, Tile::Spawn { team_side } if *team_side == side))
    }
}

#[must_use]
pub fn default_control_nodes() -> [ControlNode; DEFAULT_CONTROL_NODE_COUNT] {
    [
        ControlNode::new(
            NodeId::new("node_north_relay"),
            "North Relay",
            Position::new(7, 0),
            1,
        ),
        ControlNode::new(
            NodeId::new("node_core_terminal"),
            "Core Terminal",
            Position::new(7, 4),
            2,
        ),
        ControlNode::new(
            NodeId::new("node_south_relay"),
            "South Relay",
            Position::new(7, 8),
            1,
        ),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub match_id: crate::domain::MatchId,
    pub turn: u32,
    pub state: PublicGameState,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicGameState {
    pub match_id: crate::domain::MatchId,
    pub turn: u32,
    pub agents: Vec<crate::domain::GameAgent>,
    pub nodes: Vec<ControlNode>,
    pub score: std::collections::HashMap<TeamId, i32>,
}
