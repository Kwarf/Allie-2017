use ai::strategies::StrategyType;
use ai::{Bot, Strategy};
use common::{Direction, Position};
use game::Map;
use protocol::GameState;
use std::cmp;
use traits::HasPosition;

pub struct PickPellets {
    current_path: Vec<Position>,
}

impl PickPellets {
    pub fn new() -> PickPellets {
        PickPellets {
            current_path: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.current_path.clear();
    }
}

impl Strategy for PickPellets {
    fn description(&self) -> StrategyType {
        StrategyType::PickPellets
    }

    fn action(&mut self, bot: &Bot, state: &GameState) -> Option<Direction> {
        if bot.previous_strategy_type != Some(self.description()) {
            self.reset();
        }

        // If we have a destination we should probably keep walking there
        if let Some(next) = self.current_path.pop() {
            return state.me.position().direction_to(&next);
        }

        // With no path we keep going straight if there's pellets there
        let position_if_continue = state.me.position().adjacent(&state.map, &bot.previous_direction);
        if state.map.tile_at(&position_if_continue).is_pellet() {
            return Some(bot.previous_direction.clone());
        }

        // If there's pellets next to us, go in that direction instead
        if let Some(pos) = state.me.position()
            .neighbours(&state.map)
            .into_iter()
            .find(|p| state.map.tile_at(&p).is_pellet()) {
            return state.me.position().direction_to(&pos);
        }

        // As a last resort we pathfind to all intersections,
        // and pick the path that contains most pellets relative to its length
        let path: Option<(_, Vec<Position>)> = bot.map_information
            .turning_points()
            .into_iter()
            .map(|p| bot.path_graph.path_to(p))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap())
            .filter(|path| path.len() > 0)
            .map(|path| (state.map.points_in_path(&path) as f32 / path.len() as f32, path))
            .max_by(|&(pp1, _), &(pp2, _)| {
                pp1.partial_cmp(&pp2).unwrap_or(cmp::Ordering::Equal)
            });

        if let Some(p) = path {
            self.current_path = p.1;
            return state.me.position().direction_to(&self.current_path.pop().unwrap());
        }

        None
    }
}
