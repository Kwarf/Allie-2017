use ai::pathfinder::PathNode;
use ai::{Bot, Strategy, pathfinder};
use ai::strategies::StrategyType;
use common::{Direction, Position};
use protocol::GameState;
use std::rc::Rc;
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

        let map_state = Rc::new(state.map.clone());
        let origin_node = PathNode {
            position: state.me.position(),
            map_information: bot.map_information.clone(),
            current_map_state: map_state.clone(),
        };

        let path: Option<Vec<Position>> = state.enemies
            .iter()
            .filter(|x| !x.is_dangerous) // TODO: Also hunt people that will not be dangerous for as long as me
            .map(|x| PathNode {
                position: x.position(),
                map_information: bot.map_information.clone(),
                current_map_state: map_state.clone(),
            })
            .map(|node| pathfinder::get_shortest(&origin_node, &node))
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
