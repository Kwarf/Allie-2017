mod json;
mod message_type;

use game;

trait HasMap {
    fn map(&self) -> game::Map;
}

struct WelcomeMessage;
