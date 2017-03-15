pub mod tcp;

use protocol;

pub trait AIClient {
    fn identify_as(&mut self, name: &str);

    fn wait_response(&mut self) -> bool;
    fn response(&self) -> Result<protocol::Message, protocol::Error>;
}
