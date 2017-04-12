use std::collections::HashMap;

use ai::{Bot, Strategy};
use ai::strategies::{StrategyType, weights};
use common::{Direction, Position};
use protocol::GameState;
use traits::HasPosition;

pub struct Hunter;

impl Hunter {
    pub fn new() -> Hunter {
        Hunter { }
    }
}

impl Strategy for Hunter {
    fn description(&self) -> StrategyType {
        StrategyType::Hunter
    }

    fn action(&mut self, bot: &Bot, state: &GameState) -> HashMap<Direction, i32> {
        if !bot.can_eat_others() {
            return HashMap::new();
        }

        let path: Option<Vec<Position>> = state.enemies
            .iter()
            .filter(|x| !x.is_dangerous) // TODO: Also hunt people that will not be dangerous for as long as me
            .filter(|x| bot.map_information.is_dead_end(&x.position()))
            .map(|x| bot.path_graph.path_to(&x.position()))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap())
            .min_by(|p1, p2| {
                p1.len().cmp(&p2.len())
            });

        match path {
            Some(p) => {
                let mut weights = HashMap::new();
                weights.insert(state.me.position().direction_to(&state.map, &p.last().unwrap()).unwrap(), weights::HUNT);
                weights
            },
            None => return HashMap::new(),
        }
    }
}
