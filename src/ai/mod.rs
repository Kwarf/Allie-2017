use std::cell::RefCell;
use std::rc::Rc;

mod pathfinder;
mod strategies;

use common::{Direction, Position};
use game;
use protocol;

pub trait Strategy {
    fn action(&mut self, bot: &Bot, state: &protocol::GameState) -> Option<Direction>;
}

pub struct Bot {
    map_information: Rc<game::MapInformation>, // See PathNode in pathfinder
    strategies: Vec<RefCell<Box<Strategy>>>,

    current_destination: Option<Position>,
    previous_direction: Direction,

    tick: u32,
}

impl Bot {
    pub fn from_game_state(state: protocol::GameState) -> Bot {
        Bot {
            map_information: Rc::new(game::MapInformation::from_map(&state.map)),
            strategies: vec![
                RefCell::new(Box::new(strategies::PickPellets::new())),
            ],

            current_destination: None,
            previous_direction: Direction::Down, // Chosen by fair dice roll, https://xkcd.com/221/

            tick: 0,
        }
    }

    pub fn determine_action(&mut self, state: protocol::GameState) -> Direction {
        self.tick += 1;

        let suggested_actions = self.strategies
            .iter()
            .map(|x| x.borrow_mut().action(&self, &state))
            .collect::<Vec<Option<Direction>>>();

        let decision = suggested_actions
            .into_iter()
            .find(|x| x.is_some())
            .unwrap_or(None)
            .unwrap_or_else(|| {
                println!("FALLBACK MOVEMENT");
                self.previous_direction.clone()
            });

        if self.previous_direction != decision {
            self.previous_direction = decision.clone();
        }
        decision
    }

    pub fn reset(&mut self) {
        self.previous_direction = Direction::Down;
        self.current_destination = None;
        self.tick = 0;
    }
}
