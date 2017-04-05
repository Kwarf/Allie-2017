use std::collections::HashMap;

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
    fn action(&mut self, bot: &ai::Bot, current_state: &protocol::GameState) -> HashMap<common::Direction, i32>;
}

mod weights {
    // Tiles that may cause a collision with another player
    pub const AVOID: i32 = -100;

    // Tiles leading in the direction of someone we are hunting
    pub const HUNT: i32 = 10;

    // Tiles leading to a super pellet
    pub const KILL_SPELLET: i32 = 10;
    // Tiles leading to another player that we can kill
    pub const KILL_PLAYER: i32 = 10;

    // Tiles leading to or containing pellets
    pub const PELLET: i32 = 1;
}
