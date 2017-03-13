use serde_json;

#[derive(Debug, Deserialize)]
struct MessageBase {
    messagetype: String,
}

#[derive(Debug, Deserialize)]
struct Entity {
    id: u32,
    x: u32,
    y: u32,
    // These fields are optional as we don't receive them in the welcome message
    score: Option<u32>,
    isdangerous: Option<bool>,
}

#[derive(Deserialize)]
struct Map {
    content: Vec<String>,
    height: u32,
    width: u32,
    pelletsleft: u32,
}

#[derive(Deserialize)]
struct Welcome {
    map: Map,
    you: Entity,
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_WELCOME: &'static str = r#"
{
    "map": {
        "content": [
            "||||||||||||||||||||||||||||",
            "|............||............|",
            "|.||||.|||||.||.|||||.||||.|",
            "|o||||.|||||.||.|||||.||||o|",
            "|.||||.|||||.||.|||||.||||.|",
            "|..........................|",
            "|.||||.||.||||||||.||.||||.|",
            "|.||||.||.||||||||.||.||||.|",
            "|......||....||....||......|",
            "||||||.|||||_||_|||||.||||||",
            "_____|.|||||_||_|||||.|_____",
            "_____|.||__________||.|_____",
            "_____|.||_|||__|||_||.|_____",
            "||||||.||_|______|_||.||||||",
            "______.___|______|___.______",
            "||||||.||_|______|_||.||||||",
            "_____|.||_|||__|||_||.|_____",
            "_____|.||__________||.|_____",
            "_____|.||_||||||||_||.|_____",
            "||||||.||_||||||||_||.||||||",
            "|............||............|",
            "|.||||.|||||.||.|||||.||||.|",
            "|.||||.|||||.||.|||||.||||.|",
            "|o..||.......__.......||..o|",
            "|||.||.||.||||||||.||.||.|||",
            "|||.||.||.||||||||.||.||.|||",
            "|......||....||....||......|",
            "|.||||||||||.||.||||||||||.|",
            "|.||||||||||.||.||||||||||.|",
            "|..........................|",
            "||||||||||||||||||||||||||||"
        ],
        "height": 31,
        "pelletsleft": 240,
        "width": 28
    },
    "messagetype": "welcome",
    "you": {
        "id": 0,
        "x": 11,
        "y": 13
    }
}"#;

    #[test]
    fn can_deserialize_welcome() {
        let message_base: MessageBase = serde_json::from_str(EXAMPLE_WELCOME).unwrap();
        assert_eq!("welcome", message_base.messagetype);

        let message: Welcome = serde_json::from_str(EXAMPLE_WELCOME).unwrap();
        assert_eq!(31, message.map.content.len());
        assert_eq!(31, message.map.height);
        assert_eq!(240, message.map.pelletsleft);
        assert_eq!(28, message.map.width);

        assert_eq!(0, message.you.id);
        assert_eq!(11, message.you.x);
        assert_eq!(13, message.you.y);
        assert_eq!(None, message.you.score);
        assert_eq!(None, message.you.isdangerous);
    }
}
