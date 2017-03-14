use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Welcome,
    StateUpdate,
    Dead,
    EndOfRound,
    StartOfRound,
}

#[derive(Debug)]
pub enum MessageTypeError {
    UnknownType,
}

impl FromStr for MessageType {
    type Err = MessageTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "welcome" => Ok(MessageType::Welcome),
            "stateupdate" => Ok(MessageType::StateUpdate),
            "dead" => Ok(MessageType::Dead),
            "endofround" => Ok(MessageType::EndOfRound),
            "startofround" => Ok(MessageType::StartOfRound),
            _ => Err(MessageTypeError::UnknownType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_welcome_type() {
        assert_eq!(MessageType::Welcome, MessageType::from_str("welcome").unwrap());
    }

    #[test]
    fn can_convert_stateupdate_type() {
        assert_eq!(MessageType::StateUpdate, MessageType::from_str("stateupdate").unwrap());
    }

    #[test]
    fn can_convert_dead_type() {
        assert_eq!(MessageType::Dead, MessageType::from_str("dead").unwrap());
    }

    #[test]
    fn can_convert_endofround_type() {
        assert_eq!(MessageType::EndOfRound, MessageType::from_str("endofround").unwrap());
    }

    #[test]
    fn can_convert_startofround_type() {
        assert_eq!(MessageType::StartOfRound, MessageType::from_str("startofround").unwrap());
    }
}
