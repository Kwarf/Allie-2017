use std::rc::Rc;

mod pathfinder;

use common::{Direction, Position};
use game;
use itertools::Itertools;
use protocol;
use traits::HasPosition;

pub struct Bot {
    map_information: Rc<game::MapInformation>, // See PathNode in pathfinder

    current_path: Vec<Position>,
}

impl Bot {
    pub fn from_game_state(state: protocol::GameState) -> Bot {
        Bot {
            map_information: Rc::new(game::MapInformation::from_map(&state.map)),

            current_path: Vec::new(),
        }
    }

    pub fn determine_action(&mut self, state: protocol::GameState) -> Direction {
        if self.current_path.len() == 0 {
            let map_state = Rc::new(state.map.clone());

            let origin_node = pathfinder::PathNode {
                position: state.me.position(),
                map_information: self.map_information.clone(),
                current_map_state: map_state.clone(),
            };

            // Pathfind to all corners/intersections, to determine our route
            let mut paths: Vec<Vec<Position>> = self.map_information
                .turning_points()
                // Initial sort by manhattan distance
                .sorted_by(|p1, p2| {
                    let d1 = state.me.position().manhattan_distance_to(p1);
                    let d2 = state.me.position().manhattan_distance_to(p2);
                    d1.cmp(&d2)
                })
                .into_iter()
                .map(|pos| pathfinder::PathNode {
                    position: pos.clone(),
                    map_information: self.map_information.clone(),
                    current_map_state: map_state.clone(),
                })
                // Pathfinding will be lazy...
                .map(|node| pathfinder::get_shortest(&origin_node, &node))
                .filter(|path| path.is_some())
                .map(|path| path.unwrap())
                // ...and look for paths with points in them...
                .filter(|path| state.map.points_in_path(path) > 0)
                // ...and stop when a single one is found
                .take(1)
                .collect();

            paths.sort_by(|p1, p2| p1.len().cmp(&p2.len()));
            println!("Found {} total paths", paths.len());

            paths = paths
                .into_iter()
                .filter(|p| state.map.points_in_path(p) > 0)
                .collect();
            println!("{} of them are viable (has points)", paths.len());

            self.current_path = paths[0].clone();

            println!("Walking from {} to {}, a distance of {} steps"
                , state.me.position()
                , self.current_path[0]
                , self.current_path.len());
        }

        self.current_path
            .pop()
            .and_then(|x| state.me.position().direction_to(&x))
            .expect("Did not find a direction to walk in..")
    }

    pub fn reset(&mut self) {
        self.current_path.clear();
    }
}
