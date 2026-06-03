use crate::domain::ids::{MapId, NodeId, TeamId};
use crate::domain::team::TeamSide;
use crate::domain::error::DomainError;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn manhattan_distance(&self, other: &Self) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
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
pub struct GameMap {
    pub id: MapId,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<Tile>>,
}

impl GameMap {
    pub fn get_tile(&self, pos: &Position) -> Option<&Tile> {
        if pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32 {
            Some(&self.tiles[pos.y as usize][pos.x as usize])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlNode {
    pub id: NodeId,
    pub name: String,
    pub position: Position,
    pub score_value: i32,
    pub controlled_by: Option<TeamId>,
}

impl ControlNode {
    pub fn new(id: NodeId, name: String, position: Position, score_value: i32) -> Self {
        Self {
            id,
            name,
            position,
            score_value,
            controlled_by: None,
        }
    }
}

/// Helper to parse a text map.
/// Characters:
/// . = Normal
/// # = Wall
/// o = Cover
/// A = Spawn A
/// B = Spawn B
/// N = North Relay
/// C = Core Terminal
/// S = South Relay
pub fn parse_map(id: MapId, input: &str) -> Result<(GameMap, Vec<ControlNode>), DomainError> {
    let mut tiles = Vec::new();
    let mut nodes = Vec::new();
    let mut spawn_a_exists = false;
    let mut spawn_b_exists = false;

    let lines: Vec<&str> = input
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return Err(DomainError::MapError("Map input is empty".to_string()));
    }

    let height = lines.len() as u32;
    let mut width_opt = None;

    for (y, line) in lines.iter().enumerate() {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let row_width = tokens.len() as u32;

        if let Some(w) = width_opt {
            if row_width != w {
                return Err(DomainError::MapError(format!(
                    "Inconsistent row width at line {}: expected {}, got {}",
                    y + 1, w, row_width
                )));
            }
        } else {
            width_opt = Some(row_width);
        }

        let mut row_tiles = Vec::new();
        for (x, token) in tokens.iter().enumerate() {
            let pos = Position::new(x as i32, y as i32);
            let tile = match *token {
                "." => Tile::Normal,
                "#" => Tile::Wall,
                "o" => Tile::Cover,
                "A" => {
                    spawn_a_exists = true;
                    Tile::Spawn { team_side: TeamSide::A }
                }
                "B" => {
                    spawn_b_exists = true;
                    Tile::Spawn { team_side: TeamSide::B }
                }
                "N" => {
                    let id = NodeId::new("north-relay");
                    nodes.push(ControlNode::new(id.clone(), "North Relay".to_string(), pos, 1));
                    Tile::Node { node_id: id }
                }
                "C" => {
                    let id = NodeId::new("core-terminal");
                    nodes.push(ControlNode::new(id.clone(), "Core Terminal".to_string(), pos, 2));
                    Tile::Node { node_id: id }
                }
                "S" => {
                    let id = NodeId::new("south-relay");
                    nodes.push(ControlNode::new(id.clone(), "South Relay".to_string(), pos, 1));
                    Tile::Node { node_id: id }
                }
                other => {
                    return Err(DomainError::MapError(format!(
                        "Invalid tile character '{}' at ({}, {})",
                        other, x, y
                    )));
                }
            };
            row_tiles.push(tile);
        }
        tiles.push(row_tiles);
    }

    let width = width_opt.unwrap_or(0);
    if width != 15 || height != 9 {
        return Err(DomainError::MapError(format!(
            "Invalid map dimensions: expected 15x9, got {}x{}",
            width, height
        )));
    }

    if nodes.len() != 3 {
        return Err(DomainError::MapError(format!(
            "Expected exactly 3 control nodes, found {}",
            nodes.len()
        )));
    }

    // Check node identity
    let has_north = nodes.iter().any(|n| n.id.as_str() == "north-relay");
    let has_core = nodes.iter().any(|n| n.id.as_str() == "core-terminal");
    let has_south = nodes.iter().any(|n| n.id.as_str() == "south-relay");
    if !has_north || !has_core || !has_south {
        return Err(DomainError::MapError(
            "Map must contain exactly North Relay (N), Core Terminal (C), and South Relay (S)".to_string(),
        ));
    }

    if !spawn_a_exists || !spawn_b_exists {
        return Err(DomainError::MapError(
            "Map must contain spawn locations for both Team A (A) and Team B (B)".to_string(),
        ));
    }

    Ok((GameMap { id, width, height, tiles }, nodes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MAP: &str = r#"
        .  .  .  #  .  .  o  N  o  .  .  #  .  .  .
        A  A  .  #  .  .  .  .  .  .  .  #  .  B  B
        A  A  .  .  .  #  o  .  o  #  .  .  .  B  B
        .  .  .  o  .  .  .  .  .  .  .  o  .  .  .
        .  A  .  .  .  o  .  C  .  o  .  .  .  B  .
        .  .  .  o  .  .  .  .  .  .  .  o  .  .  .
        A  A  .  .  .  #  o  .  o  #  .  .  .  B  B
        A  A  .  #  .  .  .  .  .  .  .  #  .  B  B
        .  .  .  #  .  .  o  S  o  .  .  #  .  .  .
    "#;

    #[test]
    fn test_parse_valid_map() {
        let (map, nodes) = parse_map(MapId::new("test"), VALID_MAP).unwrap();
        assert_eq!(map.width, 15);
        assert_eq!(map.height, 9);
        assert_eq!(nodes.len(), 3);
        
        let core = nodes.iter().find(|n| n.id.as_str() == "core-terminal").unwrap();
        assert_eq!(core.position, Position::new(7, 4));
        assert_eq!(core.score_value, 2);

        assert_eq!(map.get_tile(&Position::new(7, 4)).unwrap(), &Tile::Node { node_id: NodeId::new("core-terminal") });
        assert_eq!(map.get_tile(&Position::new(0, 1)).unwrap(), &Tile::Spawn { team_side: TeamSide::A });
        assert_eq!(map.get_tile(&Position::new(13, 1)).unwrap(), &Tile::Spawn { team_side: TeamSide::B });
    }

    #[test]
    fn test_parse_invalid_dimensions() {
        let bad_map = r#"
            . . .
            A B C
        "#;
        assert!(parse_map(MapId::new("test"), bad_map).is_err());
    }

    #[test]
    fn test_manhattan_distance() {
        let p1 = Position::new(0, 0);
        let p2 = Position::new(3, 4);
        assert_eq!(p1.manhattan_distance(&p2), 7);
    }
}
