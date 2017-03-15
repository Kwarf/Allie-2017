use serde_json;

mod json;
mod message_type;

use game;

trait HasMap {
    fn map(&self) -> game::Map;
}

#[derive(Debug)]
struct GameState {
    map: Map,
    me: Player,
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Floor,
    Wall,
    Door,
    Pellet,
    SuperPellet,
}

#[derive(Debug)]
struct Map {
    tiles: Vec<Tile>,
    width: u32,
}

impl Map {
    fn tile_at(&self, x: u32, y: u32) -> Tile {
        self.tiles[(self.width * y + x) as usize]
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

#[cfg(test)]
mod tests {
    use std;
    use super::*;

    #[test]
    fn should_have_optimal_tile_enum_size() {
        // Make sure that tiles are 1B, as we store and index lots of them
        assert_eq!(1, std::mem::size_of::<Tile>());
    }
}
