use ai;
use common;
use protocol;

pub mod hunter;
pub mod pick_pellets;

pub use self::hunter::Hunter;
pub use self::pick_pellets::PickPellets;

pub trait Strategy {
    fn action(&mut self, bot: &ai::Bot, current_state: &protocol::GameState) -> Option<common::Direction>;
}
