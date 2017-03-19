use serde_json;

pub mod json;
mod message_type;

use common;
use game;
use traits;
use traits::HasDimensions;

#[derive(Debug, Deserialize)]
struct GameState {
    map: game::Map,

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
