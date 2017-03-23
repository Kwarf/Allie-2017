pub mod tcp;

use common;
use protocol;

pub trait AIClient {
    fn identify_as(&mut self, name: &str);

    fn wait_response(&mut self) -> bool;
    fn response(&self) -> Result<protocol::Message, protocol::Error>;

    fn send_action(&mut self, direction: common::Direction);
}
