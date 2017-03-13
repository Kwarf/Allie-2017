use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Welcome,
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
}
