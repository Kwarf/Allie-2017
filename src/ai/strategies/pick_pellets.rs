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

        if self.target_pellet.is_none() || !state.map.tile_at(&self.target_pellet.clone().unwrap()).is_pellet() {
            let path: Option<Vec<Position>> = pathfinder::find_closest_pellet(&state.map, &state.me.position(), &state.enemies);
            if let Some(p) = path {
                self.target_pellet = Some(p[0].clone());
            } else {
                self.target_pellet = None;
            }
        }

        if let &Some(ref pos) = &self.target_pellet {
            return bot.path_graph
                .path_to(&pos)
                .and_then(|path| Some(path.last().unwrap().clone()))
                .and_then(|pos| state.me.position().direction_to(&state.map, &pos));
        }

        None
    }
}
