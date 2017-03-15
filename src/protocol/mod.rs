use serde_json;

mod json;
mod message_type;

use game;

trait HasMap {
    fn map(&self) -> game::Map;
}

#[derive(Debug)]
struct GameState;

struct Position {
    x: i32,
    y: i32,
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
