use ai::{Bot, Strategy};
use ai::strategies::StrategyType;
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

    fn action(&mut self, bot: &Bot, state: &GameState) -> Option<Direction> {
        if !bot.can_eat_others() {
            return None;
        }

        let path: Option<Vec<Position>> = state.enemies
            .iter()
            .filter(|x| !x.is_dangerous) // TODO: Also hunt people that will not be dangerous for as long as me
            .map(|x| bot.path_graph.path_to(&x.position()))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap())
            .min_by(|p1, p2| {
                p1.len().cmp(&p2.len())
            });

        match path {
            Some(p) => state.me.position().direction_to(&p.last().unwrap()),
            None => None,
        }
    }
}
