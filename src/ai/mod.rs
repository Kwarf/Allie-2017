use rand;
use rand::Rng;

use common::Direction;
use game;
use protocol;

pub struct Bot {
    map_information: game::MapInformation,
    rng: rand::ThreadRng,
}

impl Bot {
    pub fn from_game_state(state: protocol::GameState) -> Bot {
        Bot {
            map_information: game::MapInformation::from_map(&state.map),
            rng: rand::thread_rng(),
        }
    }

    pub fn determine_action(&mut self, state: protocol::GameState) -> Direction {
        match self.rng.gen_range(0, 4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => panic!("Invalid random response"),
        }
    }
}
