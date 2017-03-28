use std::collections::HashSet;

use ai::Bot;
use ai::strategies::Strategy;
use common::{Direction, Position};
use protocol::GameState;
use traits::HasPosition;

pub struct Avoidance;

impl Avoidance {
    pub fn new() -> Avoidance {
        Avoidance { }
    }
}

impl Strategy for Avoidance {
    fn action(&mut self, bot: &Bot, state: &GameState) -> Option<Direction> {
        let neighbouring_positions: HashSet<Position> = state.me.position()
            .neighbours(&state.map)
            .into_iter()
            .collect();

        let enemy_positions: HashSet<Position> = state.enemies
            .iter()
            .filter(|e| !(bot.can_eat_others() && !e.is_dangerous))
            .map(|e| e.position())
            .collect();

        let neighbouring_enemy_directions: HashSet<Direction> = neighbouring_positions
            .intersection(&enemy_positions)
            .map(|p| state.me.position().direction_to(&p).unwrap())
            .collect();

        if neighbouring_enemy_directions.len() == 0 {
            return None; // No enemies next to me
        }

        println!("Enemy next to me, running away!");

        Direction::hash_set_all()
            .difference(&neighbouring_enemy_directions)
            .find(|d| state.map.tile_at(&state.me.position().neighbour(&state.map, d)).is_walkable())
            .and_then(|d| Some(d.clone()))
    }
}
