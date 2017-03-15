use serde;
use serde::{Deserialize, Deserializer};
use serde_json;
use std::str::FromStr;

use protocol;

#[derive(Debug, Deserialize)]
struct BaseMessage {
    messagetype: String,
}

#[derive(Deserialize)]
struct StateUpdateMessage {
    gamestate: protocol::GameState,
}

pub fn deserialize_map_content<T>(deserializer: T) -> Result<Vec<protocol::Tile>, T::Error>
    where T: Deserializer {
    let content: Vec<String> = Deserialize::deserialize(deserializer)?;
    Ok(content
        .concat()
        .chars()
        .map(|x| {
            match x {
                '_' => protocol::Tile::Floor,
                '|' => protocol::Tile::Wall,
                '-' => protocol::Tile::Door,
                '.' => protocol::Tile::Pellet,
                'o' => protocol::Tile::SuperPellet,
                _ => {
                    debug_assert!(false, "Encountered unknown tile in map, will default to Wall in release builds");
                    protocol::Tile::Wall
                },
            }
        })
        .collect())
}

impl FromStr for protocol::Message {
    type Err = protocol::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let base: Result<BaseMessage, _> = serde_json::from_str(s);
        if base.is_err() {
            return Err(protocol::Error::MissingGamestate);
        }

        // For some reason welcome and stateupdate differ in structure, requiring this ugliness
        match base.unwrap().messagetype.as_ref() {
            "welcome" => {
                let state: Result<protocol::GameState, _> = serde_json::from_str(s);
                match state {
                    Ok(x) => Ok(protocol::Message::Welcome { state: x.into() }),
                    Err(e) => Err(protocol::Error::DeserializationError(e))
                }
            },
            "stateupdate" => {
                let state: Result<StateUpdateMessage, _> = serde_json::from_str(s);
                match state {
                    Ok(x) => Ok(protocol::Message::Update { state: x.gamestate.into() }),
                    Err(e) => Err(protocol::Error::DeserializationError(e))
                }
            },
            "dead" => Ok(protocol::Message::Dead),
            "endofround" => Ok(protocol::Message::EndOfRound),
            "startofround" => Ok(protocol::Message::StartOfRound),
            _ => Err(protocol::Error::UnknownMessageType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_WELCOME: &'static str = r#"{"map":{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|..........................|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|......||....||....||......|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||__|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||__|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":240,"width":28},"messagetype":"welcome","you":{"id":0,"x":11,"y":13}}"#;
    const EXAMPLE_STATEUPDATE: &'static str = r#"{"gamestate":{"map":{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|..........................|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|......||....||....||......|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||__|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||__|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":240,"width":28},"others":[],"you":{"id":0,"x":11,"y":13,"score":130,"isdangerous":true}},"messagetype":"stateupdate"}"#;

    #[test]
    fn can_deserialize_welcome() {
        let message = protocol::Message::from_str(EXAMPLE_WELCOME).unwrap();
        match message {
            protocol::Message::Welcome { state } => {
                assert_example_map(&state.map);
                // TODO: Assert correct state
            },
            _ => { assert!(false, "Incorrect type returned") },
        }
    }

    #[test]
    fn can_deserialize_stateupdate() {
        let message = protocol::Message::from_str(EXAMPLE_STATEUPDATE).unwrap();
        match message {
            protocol::Message::Update { state } => {
                assert_example_map(&state.map);

                assert_eq!(0, state.me.id);
                assert_eq!(11, state.me.x);
                assert_eq!(13, state.me.y);
                assert_eq!(130, state.me.score);
                assert_eq!(true, state.me.is_dangerous);
            },
            _ => { assert!(false, "Incorrect type returned") },
        }
    }

    fn assert_example_map(map: &protocol::Map) {
        assert_eq!(28, map.width);
        assert_eq!(868, map.tiles.len());

        // Test tile types, randomly picked locations
        assert_eq!(protocol::Tile::Floor, map.tile_at(12, 10));
        assert_eq!(protocol::Tile::Wall, map.tile_at(0, 30));
        // assert_eq!(protocol::Tile::Door, map.tile_at()); // Example has no door :(, let's just assume it works for now
        assert_eq!(protocol::Tile::Pellet, map.tile_at(26, 1));
        assert_eq!(protocol::Tile::SuperPellet, map.tile_at(26, 3));
    }
}
