use std::rc::Rc;

mod pathfinder;

use common::{Direction, Position};
use game;
use itertools::Itertools;
use protocol;
use traits::HasPosition;

pub struct Bot {
    map_information: Rc<game::MapInformation>, // See PathNode in pathfinder

    current_destination: Option<Position>,
}

impl Bot {
    pub fn from_game_state(state: protocol::GameState) -> Bot {
        Bot {
            map_information: Rc::new(game::MapInformation::from_map(&state.map)),

            current_destination: None,
        }
    }

    pub fn determine_action(&mut self, state: protocol::GameState) -> Direction {
        let map_state = Rc::new(state.map.clone());
        let origin_node = pathfinder::PathNode {
            position: state.me.position(),
            map_information: self.map_information.clone(),
            current_map_state: map_state.clone(),
        };

        // Reset destination if we reached it
        if let Some(d) = self.current_destination.clone() {
            if state.me.position() == d {
                self.current_destination = None;
            }
        }

        if self.current_destination.is_none() {
            // Pathfind to all corners/intersections, to determine our route
            let paths: Vec<Vec<Position>> = self.map_information
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

            // Found no path by intersections, go to the closest pellet, if any
            if paths.len() == 0 {
                let pellets = state.map.pellets();
                println!("Direct-pellet fallback, {} pellets left", pellets.len());

                // Big red code-duplication warning-flags here, but yeah, will probably rewrite tomorrow
                let fallback: Vec<Vec<Position>> = pellets
                    .into_iter()
                    .sorted_by(|p1, p2| {
                        let d1 = state.me.position().manhattan_distance_to(p1);
                        let d2 = state.me.position().manhattan_distance_to(p2);
                        d1.cmp(&d2)
                    })
                    .into_iter()
                    .take(1)
                    .map(|pos| pathfinder::PathNode {
                        position: pos.clone(),
                        map_information: self.map_information.clone(),
                        current_map_state: map_state.clone(),
                    })
                    .map(|node| pathfinder::get_shortest(&origin_node, &node))
                    .filter(|path| path.is_some())
                    .map(|path| path.unwrap())
                    .collect();

                if fallback.len() > 0 {
                    println!("Found fallback path");
                    self.current_destination = Some(fallback[0][0].clone());
                }
            }

            if paths.len() > 0 {
                let path = &paths[0];
                self.current_destination = Some(path[0].clone());

                println!("Walking from {} to {}, a distance of {} steps"
                    , state.me.position()
                    , path[0]
                    , path.len());
            }
        }

        if let Some(d) = self.current_destination.clone() {
            let target_node = pathfinder::PathNode {
                position: d,
                map_information: self.map_information.clone(),
                current_map_state: map_state.clone(),
            };

            if let Some(p) = pathfinder::get_shortest(&origin_node, &target_node) {
                println!("{} steps left to target", p.len());
                return state.me.position().direction_to(&p.last().unwrap()).unwrap();
            }
        }

        Direction::Down // TODO: Something better when we could not find a direction to walk in..
    }

    pub fn reset(&mut self) {
        self.current_destination = None;
    }
}
