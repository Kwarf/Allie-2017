mod json;
mod message_type;

use game;

trait HasMap {
    fn map(&self) -> game::Map;
}

#[derive(Debug)]
struct GameState;

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
    DeserializationError,
}
