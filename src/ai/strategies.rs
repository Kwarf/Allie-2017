use ai::pathfinder::PathNode;
use ai::{Bot, Strategy, pathfinder};
use common::{Direction, Position};
use game::Map;
use protocol::GameState;
use std::rc::Rc;
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
}

impl Strategy for PickPellets {
    fn action(&mut self, bot: &Bot, state: &GameState) -> Option<Direction> {
        let map_state = Rc::new(state.map.clone());
        let origin_node = PathNode {
            position: state.me.position(),
            map_information: bot.map_information.clone(),
            current_map_state: map_state.clone(),
        };

        // If we have a destination we should probably keep walking there
        if let Some(next) = self.current_path.pop() {
            return state.me.position().direction_to(&next);
        }

        // With no path we keep going straight if there's pellets there
        let position_if_continue = state.me.position().neighbour::<Map>(&state.map, &bot.previous_direction);
        if state.map.tile_at(&position_if_continue).is_pellet() {
            return Some(bot.previous_direction.clone());
        }

        // If there's pellets next to us, go in that direction instead
        if let Some(pos) = state.me.position()
            .neighbours::<Map>(&state.map)
            .into_iter()
            .find(|p| state.map.tile_at(&p).is_pellet()) {
            return state.me.position().direction_to(&pos);
        }

        // As a last resort we pathfind to all intersections,
        // and pick the path that contains most pellets
        let path: Option<Vec<Position>> = bot.map_information
            .turning_points()
            .into_iter()
            .map(|pos| PathNode {
                position: pos.clone(),
                map_information: bot.map_information.clone(),
                current_map_state: map_state.clone(),
            })
            .map(|node| pathfinder::get_shortest(&origin_node, &node))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap())
            .max_by(|p1, p2| {
                let pp1 = state.map.points_in_path(p1);
                let pp2 = state.map.points_in_path(p2);
                pp1.cmp(&pp2)
            });

        if let Some(p) = path {
            self.current_path = p;
            return state.me.position().direction_to(&self.current_path.pop().unwrap());
        }

        None
    }
}
