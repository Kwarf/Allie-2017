use std::collections::HashSet;

use ai::{Bot, pathfinder};
use ai::strategies::{Strategy, StrategyType};
use common::Direction;
use protocol::GameState;
use traits::HasPosition;

pub struct Avoidance;

impl Avoidance {
    pub fn new() -> Avoidance {
        Avoidance { }
    }
}

impl Strategy for Avoidance {
    fn description(&self) -> StrategyType {
        StrategyType::Avoidance
    }

    fn action(&mut self, bot: &Bot, state: &GameState) -> Option<Direction> {
        let my_position = state.me.position();

        // Get out of dead ends if we might get blocked in
        if bot.map_information.is_dead_end(&my_position) {
            let path_to_exit = bot.map_information.path_to_dead_end_exit(&my_position).unwrap();
            let dead_end_exit = path_to_exit[0].clone();

            let closest_enemy_distance_to_exit = state.enemies
                .iter()
                // Ignore any enemies in dead ends, because
                .filter(|e| !bot.map_information.is_dead_end(&e.position()))
                .map(|e| pathfinder::get_shortest(&state.map, &e.position(), &dead_end_exit).unwrap())
                .map(|path| path.len())
                .max()
                .unwrap_or(usize::max_value());

            if path_to_exit.len() + 4 > closest_enemy_distance_to_exit {
                // Time to get out
                // println!("GETTING OUT OF DEAD END: My distance to exit is {} and the closest enemy can reach that in {} steps"
                //     , path_to_exit.len()
                //     , closest_enemy_distance_to_exit);
                return state.me.position().direction_to(&state.map, &path_to_exit.last().unwrap());
            }
        }

        if bot.tick < 5 {
            return None;
        }

        let directions_to_avoid = state.enemies
            .iter()
            .filter(|e| !(bot.can_eat_others() && !e.is_dangerous))
            .map(|e| (bot.path_graph.cost_to(&e.position()).unwrap_or(usize::max_value()), e))
            .filter(|&(c, _)| c <= 3)
            .map(|(_, e)| bot.path_graph.path_to(&e.position()).unwrap().last().unwrap().clone())
            .map(|pos| state.me.position().direction_to(&state.map, &pos))
            .collect::<Option<HashSet<Direction>>>();

        if let Some(enemy_directions) = directions_to_avoid {
            if enemy_directions.len() == 0 {
                return None;
            }

            let possible_directions = Direction::hash_set_all()
                .into_iter()
                .filter(|d| state.map.tile_at(&state.me.position().adjacent(&state.map, &d)).is_walkable())
                .filter(|d| bot.map_information.is_dead_end(&state.me.position()) || !bot.map_information.is_dead_end(&state.me.position().adjacent(&state.map, &d)))
                .filter(|d| !enemy_directions.contains(d))
                .collect::<Vec<Direction>>();

            if let Some(optimal_direction) = possible_directions
                .iter()
                .map(|d| {
                    let p = state.me.position().adjacent(&state.map, &d);
                    let dtp = pathfinder::distance_to_closest_pellet(&state.map, &p, &state.enemies, |p| !bot.map_information.is_dead_end(p));
                    (dtp, d)
                })
                .min_by(|&(d1, _), &(d2, _)| d1.cmp(&d2))
                .map(|(_, d)| d) {
                return Some(optimal_direction.clone());
            }

            if possible_directions.len() > 0 {
                return Some(possible_directions[0].clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol;
    use protocol::Message;
    use std::str::FromStr;

    #[test]
    fn can_avoid_multiple_at_once() {
        const STATE: &'static str = r#"{"gamestate":{"map":{"content":["||||||||||||||||||||||||||||","|____________||____________|","|_||||_|||||_||_|||||_||||_|","|_||||_|||||_||_|||||_||||_|","|.||||_|||||_||_|||||_||||.|","|....|________________|....|","|.||||_||_||||||||_||_||||.|","|.||||_||_||||||||_||_||||.|","|....|_||____||____||_|....|","||||||_|||||_||_|||||_||||||","_____|_|||||_||_|||||_|_____","_____|_||__________||_|_____","_____|_||_|||--|||_||_|_____","||||||_||_|______|_||_||||||","__________|______|__________","||||||_||_|______|_||_||||||","_____|_||_|||--|||_||_|_____","_____|_||__________||_|_____","_____|_||_||||||||_||_|_____","||||||_||_||||||||_||_||||||","|....|_______||_______|....|","|.||||_|||||_||_|||||_||||.|","|.||||_|||||_||_|||||_||||.|","|o..||________________||..o|","|||.||_||_||||||||_||_||.|||","|||.||_||_||||||||_||_||.|||","|______||____||____||______|","|_||||||||||_||_||||||||||_|","|_||||||||||_||_||||||||||_|","|__________________________|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":42,"width":28},"others":[{"id":0,"isdangerous":true,"score":59,"x":17,"y":5},{"id":1,"isdangerous":true,"score":59,"x":12,"y":5}],"you":{"id":1,"isdangerous":false,"score":153,"x":15,"y":5}},"messagetype":"stateupdate"}"#;
        let message = protocol::Message::from_str(STATE).unwrap();

        match message {
            Message::Update { state } => {
                let mut bot = Bot::from_game_state(&state);
                bot.path_graph.update_from_map(&state.map, &state.me.position());

                let mut strategy = Avoidance::new();
                let direction = strategy.action(&bot, &state);
                assert_eq!(Direction::Up, direction.unwrap());
            }
            _ => panic!(),
        }
    }
}
