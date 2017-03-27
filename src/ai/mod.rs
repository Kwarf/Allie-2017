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

        // Reset destination if we reached it
        if let Some(d) = self.current_destination.clone() {
            if state.me.position() == d {
                self.current_destination = None;
            }
        }

        if self.current_destination.is_none() {
            // Pathfind to all corners/intersections, to determine our route
            let paths: Vec<(Vec<Position>, usize)> = self.map_information
                .turning_points()
                .iter()
                // Initial sort by manhattan distance
                .sorted_by(|p1, p2| {
                    let d1 = state.me.position().manhattan_distance_to(p1);
                    let d2 = state.me.position().manhattan_distance_to(p2);
                    d1.cmp(&d2)
                })
                .into_iter()
                // Pathfinding will be lazy...
                .map(|target| pathfinder::get_shortest(&state.me.position(), &target, &self.map_information, &map_state))
                .filter(|path| path.is_some())
                .map(|path| path.unwrap())
                // ...and look for paths with points in them...
                .filter(|path| state.map.points_in_path(&path.0) > 0)
                // ...and stop when a single one is found
                .take(1)
                .collect();

            // Found no path by intersections, go to the closest pellet, if any
            if paths.len() == 0 {
                let pellets = state.map.pellets();
                println!("Direct-pellet fallback, {} pellets left", pellets.len());

                // Big red code-duplication warning-flags here, but yeah, will probably rewrite tomorrow
                let fallback: Vec<(Vec<Position>, usize)> = pellets
                    .into_iter()
                    .sorted_by(|p1, p2| {
                        let d1 = state.me.position().manhattan_distance_to(p1);
                        let d2 = state.me.position().manhattan_distance_to(p2);
                        d1.cmp(&d2)
                    })
                    .into_iter()
                    .take(1)
                    .map(|target| pathfinder::get_shortest(&state.me.position(), &target, &self.map_information, &map_state))
                    .filter(|path| path.is_some())
                    .map(|path| path.unwrap())
                    .collect();

                if fallback.len() > 0 {
                    println!("Found fallback path");
                    self.current_destination = Some(fallback[0].0[0].clone());
                }
            }

            if paths.len() > 0 {
                let path = &paths[0];
                self.current_destination = Some(path.0[0].clone());

                println!("Walking from {} to {}, a distance of {} steps"
                    , state.me.position()
                    , path.0[0]
                    , path.0.len());
            }
        }

        if let Some(d) = self.current_destination.clone() {
            if let Some(p) = pathfinder::get_shortest(&state.me.position(), &d, &self.map_information, &map_state) {
                println!("Walking from {} to {}", p.0.last().unwrap(), p.0[0]);
                println!("Sequence:");
                for n in &p.0 {
                    println!("\t{}", n);
                }

                println!("{} cost to target", p.1);
                return state.me.position().direction_to(&p.0.last().unwrap()).unwrap();
            }
        }

        Direction::Down // TODO: Something better when we could not find a direction to walk in..
    }

    pub fn reset(&mut self) {
        self.current_destination = None;
    }
}
