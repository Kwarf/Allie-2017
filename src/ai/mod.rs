use std::cell::RefCell;
use std::collections::HashMap;

mod pathfinder;
mod strategies;

use ai::strategies::Strategy;
use common::{Direction, Position, rules};
use game;
use protocol;
use traits::HasPosition;

pub struct Bot {
    map_information: game::MapInformation,
    path_graph: pathfinder::LocalPathGraph,

    strategies: Vec<RefCell<Box<Strategy>>>,

    previous_state: Option<protocol::GameState>,

    expected_tile_type: game::TileType,
    current_destination: Option<Position>,
    previous_direction: Direction,

    tick: u32,
    remaining_ticks_dangerous: u32,
}

impl Bot {
    pub fn from_game_state(state: protocol::GameState) -> Bot {
        Bot {
            map_information: game::MapInformation::from_map(&state.map),
            path_graph: pathfinder::LocalPathGraph::new(&state.map),

            strategies: vec![
                RefCell::new(Box::new(strategies::Avoidance::new())),
                RefCell::new(Box::new(strategies::Hunter::new())),
                RefCell::new(Box::new(strategies::PickPellets::new())),
            ],

            previous_state: None,

            expected_tile_type: game::TileType::Floor,
            current_destination: None,
            previous_direction: Direction::Down, // Chosen by fair dice roll, https://xkcd.com/221/

            tick: 0,
            remaining_ticks_dangerous: 0,
        }
    }

    pub fn determine_action(&mut self, state: protocol::GameState) -> Direction {
        self.tick += 1;

        // Run BFS on map to get pathing information
        self.path_graph.update_from_map(&state.map, &state.me.position());

        // Set some state based on what tile we landed on
        if self.expected_tile_type == game::TileType::SuperPellet {
            debug_assert!(state.me.is_dangerous);
            self.remaining_ticks_dangerous = rules::TICKS_DANGEROUS + 1;
        }

        // Some asserts that our internal state matches what the server sends
        debug_assert_eq!(state.me.is_dangerous, self.can_eat_others());

        let decision = self.strategies
            .iter()
            .map(|x| x.borrow_mut().action(&self, &state))
            // .map(|x| {
            //     let weights = x.borrow_mut().action(&self, &state);
            //     println!("{:?}: {:?}", x.borrow().description(), weights);
            //     weights
            // })
            .fold(HashMap::new(), |mut acc, x| {
                for (d, f) in x {
                    let new_fitness = match acc.contains_key(&d) {
                        true => acc[&d] + f,
                        false => f,
                    };
                    acc.insert(d, new_fitness);
                }
                acc
            })
            .into_iter()
            .max_by(|d1, d2| d1.1.cmp(&d2.1))
            .map(|(d, _)| d)
            .unwrap_or(self.previous_direction.clone());

        if self.previous_direction != decision {
            self.previous_direction = decision.clone();
        }

        self.expected_tile_type = state.map.tile_at(&state.me.position().adjacent(&state.map, &decision));
        self.previous_state = Some(state);

        self.remaining_ticks_dangerous = self.remaining_ticks_dangerous.saturating_sub(1);

        decision
    }

    pub fn reset(&mut self) {
        self.previous_state = None;
        self.current_destination = None;
        self.previous_direction = Direction::Down;
        self.tick = 0;
        self.remaining_ticks_dangerous = 0;
    }

    pub fn can_eat_others(&self) -> bool {
        self.remaining_ticks_dangerous > 0
    }
}
