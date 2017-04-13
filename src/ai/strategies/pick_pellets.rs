use std::cmp;
use std::collections::HashSet;

use ai::strategies::StrategyType;
use ai::{Bot, pathfinder, Strategy};
use common::{Direction, Position};
use protocol::GameState;
use traits::HasPosition;

pub struct PickPellets {
    target_pellet: Option<Position>,
}

impl PickPellets {
    pub fn new() -> PickPellets {
        PickPellets {
            target_pellet: None,
        }
    }

    fn is_enemy_nearby(&self, bot: &Bot, state: &GameState) -> bool {
        state.enemies
            .iter()
            .filter(|e| !bot.map_information.is_dead_end(&e.position()))
            .find(|e| bot.path_graph.cost_to(&e.position()).unwrap_or(usize::max_value()) <= 3)
            .is_some()
    }
}

impl Strategy for PickPellets {
    fn description(&self) -> StrategyType {
        StrategyType::PickPellets
    }

    fn action(&mut self, bot: &Bot, state: &GameState) -> Option<Direction> {
        // We keep going straight if there's pellets there
        let position_if_continue = state.me.position().adjacent(&state.map, &bot.previous_direction);
        if state.map.tile_at(&position_if_continue).is_pellet() {
            if !bot.map_information.is_dead_end(&position_if_continue)
                || !self.is_enemy_nearby(bot, state) {
                self.target_pellet = None;
                return Some(bot.previous_direction.clone());
            }
        }

        // If there's pellets next to us, go in that direction instead
        if let Some(pos) = state.me.position()
            .neighbours(&state.map)
            .into_iter()
            .find(|p| state.map.tile_at(&p).is_pellet()) {
            if !bot.map_information.is_dead_end(&pos)
                || !self.is_enemy_nearby(bot, state) {
                self.target_pellet = None;
                return state.me.position().direction_to(&state.map, &pos);
            }
        }

        // Re-evaluate where we're going if some other strategy has been active
        if bot.previous_strategy_type != Some(self.description()) {
            self.target_pellet = None;
        }

        // Re-evaluate on each intersection
        if bot.map_information.is_intersection(&state.me.position()) {
            self.target_pellet = None;
        }

        let enemy_positions: HashSet<Position> = state.enemies
            .iter()
            .map(|x| x.position())
            .collect();

        // Try to find the optimal path not containing any enemies
        if let Some(path) = bot.map_information
            .intersections()
            .into_iter()
            .map(|p| bot.path_graph.path_to(p))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap())
            .filter(|path| path.len() > 0)
            .filter(|path| !path.iter().any(|pos| enemy_positions.contains(pos) || bot.map_information.is_dead_end(&pos)))
            .map(|path| (state.map.points_in_path(&path) as f32 / path.len() as f32, path))
            .max_by(|&(pp1, _), &(pp2, _)| {
                pp1.partial_cmp(&pp2).unwrap_or(cmp::Ordering::Equal)
            })
            .map(|(_, path)| path) {
            self.target_pellet = Some(path[0].clone())
        }

        if self.target_pellet.is_none() || !state.map.tile_at(&self.target_pellet.clone().unwrap()).is_pellet() {
            let path: Option<Vec<Position>> = pathfinder::find_closest_pellet(&state.map, &state.me.position(), &state.enemies, |_| true);
            if let Some(p) = path {
                self.target_pellet = Some(p[0].clone());
            } else {
                self.target_pellet = None;
            }
        }

        if let &Some(ref pos) = &self.target_pellet {
            return pathfinder::get_shortest_no_enemies(&state.map, &state.me.position(), &pos, &state.enemies)
                .and_then(|path| Some(path.last().unwrap().clone()))
                .and_then(|pos| state.me.position().direction_to(&state.map, &pos));
        }

        None
    }
}
