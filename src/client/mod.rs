pub mod tcp;

pub trait AIClient {
    fn identify_as(&mut self, name: &str);

    fn wait_response(&mut self) -> bool;
    fn response(&self) -> &str;
}
