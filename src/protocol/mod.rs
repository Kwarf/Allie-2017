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

struct Player {
    id: i32,
    position: Position,
    score: i32,
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
