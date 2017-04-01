use ai::Bot;
use ai::strategies::{Strategy, StrategyType};
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

    fn action(&mut self, bot: &Bot, state: &GameState) -> Option<Direction> {
        state.enemies
            .iter()
            .filter(|e| !(bot.can_eat_others() && !e.is_dangerous))
            .map(|e| (bot.path_graph.cost_to(&e.position()).unwrap(), e))
            .filter(|&(c, _)| c <= 2)
            .min_by(|&(c1, _), &(c2, _)| c1.cmp(&c2))
            .map(|(_, e)| bot.path_graph.path_to(&e.position()).unwrap().last().unwrap().clone())
            .and_then(|pos| {
                let mut possible_directions = Direction::hash_set_all();
                possible_directions.take(&state.me.position().direction_to(&state.map, &pos).unwrap());
                Some(possible_directions)
            })
            .and_then(|directions| {
                directions
                    .into_iter()
                    .find(|d| state.map.tile_at(&state.me.position().adjacent(&state.map, d)).is_walkable())
            })
    }
}
