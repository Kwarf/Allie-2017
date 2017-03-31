use std::collections::HashSet;

use ai::strategies::StrategyType;
use ai::{Bot, Strategy};
use common::{Direction, Position};
use protocol::GameState;
use std::cmp;
use traits::HasPosition;

pub struct PickPellets;

impl PickPellets {
    pub fn new() -> PickPellets {
        PickPellets { }
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
            return Some(bot.previous_direction.clone());
        }

        // If there's pellets next to us, go in that direction instead
        if let Some(pos) = state.me.position()
            .neighbours(&state.map)
            .into_iter()
            .find(|p| state.map.tile_at(&p).is_pellet()) {
            return state.me.position().direction_to(&state.map, &pos);
        }

        let enemy_positions: HashSet<Position> = state.enemies
            .iter()
            .map(|x| x.position())
            .collect();

        // As a last resort we pathfind to all intersections,
        // and pick the path that contains most pellets relative to its length
        let path: Option<Vec<Position>> = bot.map_information
            .turning_points()
            .into_iter()
            .map(|p| bot.path_graph.path_to(p))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap())
            .filter(|path| path.len() > 0)
            .filter(|path| !path.iter().any(|pos| enemy_positions.contains(pos))) // Avoid paths with enemies on, doesn't seem that great
            .map(|path| (state.map.points_in_path(&path) as f32 / path.len() as f32, path))
            .max_by(|&(pp1, _), &(pp2, _)| {
                pp1.partial_cmp(&pp2).unwrap_or(cmp::Ordering::Equal)
            })
            .map(|(_, path)| path);

        path.and_then(|p| state.me.position().direction_to(&state.map, p.last().unwrap()))
    }
}
