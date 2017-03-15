use serde_json;
use std::str::FromStr;

use protocol;

#[derive(Debug, Deserialize)]
struct BaseMessage {
    messagetype: String,
}

#[derive(Deserialize)]
struct WelcomeMessage {
    map: Map,
    you: Entity,
}

#[derive(Deserialize)]
struct StateUpdateMessage {
    gamestate: GameState,
}

#[derive(Debug, Deserialize)]
struct Entity {
    id: u32,
    x: u32,
    y: u32,
    // These fields are not present in the welcome message, default them in that case
    #[serde(default)]
    score: u32,
    #[serde(default)]
    isdangerous: bool,
}

#[derive(Debug, Deserialize)]
struct Map {
    content: Vec<String>,
    height: u32,
    width: u32,
    pelletsleft: u32,
}

#[derive(Debug, Deserialize)]
struct GameState {
    map: Map,
    others: Vec<Entity>,
    you: Entity,
}

impl FromStr for protocol::Message {
    type Err = protocol::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ => Err(protocol::Error::DeserializationError),
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
        let base: BaseMessage = serde_json::from_str(EXAMPLE_WELCOME).unwrap();
        assert_eq!("welcome", base.messagetype);

        let message: WelcomeMessage = serde_json::from_str(EXAMPLE_WELCOME).unwrap();
        assert_eq!(31, message.map.content.len());
        assert_eq!(31, message.map.height);
        assert_eq!(240, message.map.pelletsleft);
        assert_eq!(28, message.map.width);

        assert_eq!(0, message.you.id);
        assert_eq!(11, message.you.x);
        assert_eq!(13, message.you.y);
        assert_eq!(0, message.you.score);
        assert_eq!(false, message.you.isdangerous);
    }

    #[test]
    fn can_deserialize_stateupdate() {
        let base: BaseMessage = serde_json::from_str(EXAMPLE_STATEUPDATE).unwrap();
        assert_eq!("stateupdate", base.messagetype);

        let message: StateUpdateMessage = serde_json::from_str(EXAMPLE_STATEUPDATE).unwrap();
        assert_eq!(31, message.gamestate.map.content.len());
        assert_eq!(31, message.gamestate.map.height);
        assert_eq!(240, message.gamestate.map.pelletsleft);
        assert_eq!(28, message.gamestate.map.width);

        assert_eq!(0, message.gamestate.you.id);
        assert_eq!(11, message.gamestate.you.x);
        assert_eq!(13, message.gamestate.you.y);
        assert_eq!(130, message.gamestate.you.score);
        assert_eq!(true, message.gamestate.you.isdangerous);
    }
}
