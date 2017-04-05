use std::collections::HashMap;

use ai::Bot;
use ai::strategies::{Strategy, StrategyType, weights};
use common::Direction;
use protocol::GameState;
use traits::HasPosition;

pub struct Avoidance;

impl Avoidance {
    pub fn new() -> Avoidance {
        Avoidance { }
    }
}

impl Strategy for Avoidance {
    fn description(&self) -> StrategyType {
        StrategyType::Avoidance
    }

    fn action(&mut self, bot: &Bot, state: &GameState) -> HashMap<Direction, i32> {
        state.enemies
            .iter()
            .filter(|e| !(bot.can_eat_others() && !e.is_dangerous))
            .map(|e| (bot.path_graph.cost_to(&e.position()).unwrap(), e))
            .filter(|&(c, _)| c <= 2)
            .map(|(_, e)| bot.path_graph.path_to(&e.position()).unwrap().last().unwrap().clone())
            .map(|pos| (state.me.position().direction_to(&state.map, &pos).unwrap(), weights::AVOID))
            .collect()
    }
}
