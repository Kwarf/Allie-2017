use itertools::Itertools;

use ai::strategies::StrategyType;
use ai::{Bot, Strategy, pathfinder};
use common::{Direction, Position};
use game::Map;
use protocol::{GameState, Player};
use traits::HasPosition;

pub struct Killer;

impl Killer {
    pub fn new() -> Killer {
        Killer { }
    }

    fn shortest_enemy_path_cost(map: &Map, enemies: &[Player], to: &Position) -> usize {
        enemies
            .iter()
            .map(|e| pathfinder::get_shortest(map, &e.position(), to))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap().len())
            .min()
            .and_then(|d| {
                println!("Closest enemy distance to {} is {}", to, d);
                Some(d)
            })
            .unwrap_or(usize::max_value())
    }
}

impl Strategy for Killer {
    fn description(&self) -> StrategyType {
        StrategyType::Killer
    }

    fn action(&mut self, bot: &Bot, state: &GameState) -> Option<Direction> {
        let remaining_super_pellets = state.map.super_pellets();
        if !bot.can_eat_others() && remaining_super_pellets.len() == 0 {
            return None;
        }

        let path: Option<Vec<Position>> = state.enemies
            .iter()
            .filter(|x| !x.is_dangerous) // TODO: Also hunt people that will not be dangerous for as long as me
            .map(|x| bot.path_graph.path_to(&x.position()))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap())
            .filter(|path| path.len() < bot.remaining_ticks_dangerous as usize)
            .min_by(|p1, p2| {
                p1.len().cmp(&p2.len())
            });

        let distance_to_closest_eatable_player = path
            .as_ref()
            .and_then(|x| Some(x.len() as u32))
            .unwrap_or(u32::max_value());

        if !bot.can_eat_others() || bot.remaining_ticks_dangerous < distance_to_closest_eatable_player {
            // Find the closest super pellet we can reach before anyone else
            let path_to_super_pellet = state.map
                .super_pellets()
                .iter()
                .map(|pos| (bot.path_graph.cost_to(&pos), pos))
                .filter(|&(cost, _)| cost.is_some())
                .map(|(cost, pos)| (cost.unwrap(), pos))
                .sorted_by(|&(c1, _), &(c2, _)| c1.cmp(&c2))
                .into_iter()
                .find(|&(cost, pos)| cost < Killer::shortest_enemy_path_cost(&state.map, &state.enemies, pos))
                .and_then(|(_, pos)| bot.path_graph.path_to(&pos));

            return match path_to_super_pellet {
                None => None,
                Some(path) => {
                    println!("I found a super pellet at {} that I can get to in {} ticks", path[0], path.len());
                    state.me.position().direction_to(&path.last().unwrap())
                },
            }
        }

        match path {
            Some(p) => state.me.position().direction_to(&p.last().unwrap()),
            None => None,
        }
    }
}
