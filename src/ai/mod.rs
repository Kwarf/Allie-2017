use std::rc::Rc;

mod pathfinder;

use common::{Direction, Position};
use game;
use itertools::Itertools;
use protocol;
use traits::HasPosition;

pub struct Bot {
    map_information: Rc<game::MapInformation>, // See PathNode in pathfinder

    previous_direction: Direction,
    current_destination: Option<Position>,
}

impl Bot {
    pub fn from_game_state(state: protocol::GameState) -> Bot {
        Bot {
            map_information: Rc::new(game::MapInformation::from_map(&state.map)),

            previous_direction: Direction::Down, // Chosen by fair dice roll, https://xkcd.com/221/
            current_destination: None,
        }
    }

    pub fn determine_action(&mut self, state: protocol::GameState) -> &Direction {
        // If there's pellets in the direction we're travelling, just keep going
        let position_if_continue = state.me.position().neighbour::<game::Map>(&state.map, &self.previous_direction);
        if state.map.tile_at(&position_if_continue).is_pellet() {
            // It's important that we push our target with us, or we'll go back when out of pellets
            self.current_destination = Some(position_if_continue);
            return &self.previous_direction;
        }

        // If there's a pellet next to us take that
        let pellet_position = state.me.position()
            .neighbours::<game::Map>(&state.map)
            .into_iter()
            .find(|p| state.map.tile_at(&p).is_pellet());

        if let Some(pos) = pellet_position {
            self.current_destination = Some(pos.clone());
            return self.update_direction(state.me.position().direction_to(&pos).unwrap());
        }


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
            // Find the position of the closest pellet (breadth-first search)
            if let Some(path) = pathfinder::find_closest_pellet(&origin_node) {
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
                return self.update_direction(state.me.position().direction_to(&p.last().unwrap()).unwrap());
            }
        }

        println!("FALLBACK MOVEMENT");
        &self.previous_direction // TODO: Something better when we could not find a direction to walk in..
    }

    pub fn reset(&mut self) {
        self.current_destination = None;
    }

    fn update_direction(&mut self, direction: Direction) -> &Direction {
        self.previous_direction = direction;
        &self.previous_direction
    }
}
