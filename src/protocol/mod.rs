use serde_json;

mod json;
mod message_type;

use common;
use game;
use traits;
use traits::HasDimensions;

#[derive(Debug, Deserialize)]
struct GameState {
    map: Map,

    #[serde(rename = "you")]
    me: Player,

    // Only present in stateupdate messages
    #[serde(default, rename = "others")]
    enemies: Vec<Player>,
}

#[derive(Debug, Deserialize)]
struct Player {
    id: u32,
    x: u32,
    y: u32,

    // These fields are not present in the welcome message, default them in that case
    #[serde(default)]
    score: u32,
    #[serde(default, rename = "isdangerous")]
    is_dangerous: bool,
}

#[derive(Debug, Deserialize)]
pub struct Map {
    #[serde(rename = "content", deserialize_with = "json::deserialize_map_content")]
    tiles: Vec<game::TileType>,
    width: u32,
}

impl Map {
    pub fn tile_at(&self, x: u32, y: u32) -> game::TileType {
        self.tiles[(self.width * y + x) as usize]
    }

    pub fn neighbours(&self, x: u32, y: u32) -> Vec<(common::Direction, game::TileType)> {
        let mut neighbours = Vec::new();
        if x > 0 {
            neighbours.push((common::Direction::Left, self.tile_at(x - 1, y)));
        }
        if x < self.width - 1 {
            neighbours.push((common::Direction::Right, self.tile_at(x + 1, y)));
        }
        if y > 0 {
            neighbours.push((common::Direction::Up, self.tile_at(x, y - 1)));
        }
        if y < self.height() - 1 {
            neighbours.push((common::Direction::Down, self.tile_at(x, y + 1)));
        }
        neighbours
    }
}

impl traits::HasDimensions for Map {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.tiles.len() as u32 / self.width
    }
}

#[derive(Debug)]
pub enum Message {
    Welcome { state: GameState },
    Update { state: GameState },
    Dead,
    EndOfRound,
    StartOfRound,
}

#[derive(Debug)]
pub enum Error {
    MissingGamestate,
    UnknownMessageType,
    DeserializationError(serde_json::error::Error),
}
