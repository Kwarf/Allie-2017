use serde_json;

pub mod json;
mod message_type;

use common::Position;
use game;
use traits::HasPosition;

#[derive(Debug, Deserialize)]
pub struct GameState {
    pub map: game::Map,

    #[serde(rename = "you")]
    pub me: Player,

    // Only present in stateupdate messages
    #[serde(default, rename = "others")]
    enemies: Vec<Player>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    id: u32,
    x: u32,
    y: u32,

    // These fields are not present in the welcome message, default them in that case
    #[serde(default)]
    score: u32,
    #[serde(default, rename = "isdangerous")]
    is_dangerous: bool,
}

impl HasPosition for Player {
    fn position(&self) -> Position {
        Position {
            x: self.x,
            y: self.y,
        }
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
