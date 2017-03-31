use ai;
use common;
use protocol;

pub mod avoidance;
pub mod hunter;
pub mod killer;
pub mod pick_pellets;

pub use self::avoidance::Avoidance;
pub use self::hunter::Hunter;
pub use self::killer::Killer;
pub use self::pick_pellets::PickPellets;

#[derive(Debug, PartialEq)]
pub enum StrategyType {
    Avoidance,
    Hunter,
    Killer,
    PickPellets,
}

pub trait Strategy {
    fn description(&self) -> StrategyType;
    fn action(&mut self, bot: &ai::Bot, current_state: &protocol::GameState) -> Option<common::Direction>;
}
