use std::collections::HashMap;

use ai::{Bot, pathfinder};
use ai::strategies::{Strategy, StrategyType, weights};
use common::Direction;
use protocol::GameState;
use traits::HasPosition;

pub struct Avoidance {
    turn: u32,
}

impl Avoidance {
    pub fn new() -> Avoidance {
        Avoidance {
            turn: 0,
        }
    }
}

impl Strategy for Avoidance {
    fn description(&self) -> StrategyType {
        StrategyType::Avoidance
    }

    fn action(&mut self, bot: &Bot, state: &GameState) -> HashMap<Direction, i32> {
        let my_position = state.me.position();
        let mut weights = HashMap::new();

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
                let d = state.me.position().direction_to(&state.map, &path_to_exit.last().unwrap()).unwrap();
                let w = weights.entry(d).or_insert(0);
                *w += weights::EXIT_DEAD_END;
            }
        } else {
            // Don't enter dead ends with enemies nearby
            let neighbouring_dead_end = my_position.neighbours(&state.map)
                .into_iter()
                .find(|p| bot.map_information.is_dead_end(&p));
            if let Some(p) = neighbouring_dead_end {
                let nearby_enemy = state.enemies
                    .iter()
                    .filter(|e| !bot.map_information.is_dead_end(&e.position()))
                    .find(|e| bot.path_graph.cost_to(&e.position()).unwrap_or(usize::max_value()) < 4);

                if let Some(e) = nearby_enemy {
                    println!("Avoiding entering dead end at {}", p);
                    {
                        let d = state.me.position().direction_to(&state.map, &p).unwrap();
                        let w = weights.entry(d).or_insert(0);
                        *w += weights::AVOID_DEAD_END;
                    }

                    {
                        let direction_to_enemy = state.me.position().direction_to(&state.map, bot.path_graph.path_to(&e.position()).unwrap().last().unwrap()).unwrap();
                        let w = weights.entry(direction_to_enemy).or_insert(0);
                        *w += weights::AVOID_COLLIDING;
                    }
                }
            }
        }

        // This is so that we get the hell out of spawn
        self.turn += 1;
        if self.turn < 4 {
            return weights;
        }

        let directions_to_avoid = state.enemies
            .iter()
            .filter(|e| !(bot.can_eat_others() && !e.is_dangerous))
            .map(|e| (bot.path_graph.cost_to(&e.position()).unwrap(), e))
            .filter(|&(c, _)| c <= 3)
            .map(|(_, e)| bot.path_graph.path_to(&e.position()).unwrap().last().unwrap().clone())
            .map(|pos| state.me.position().direction_to(&state.map, &pos).unwrap());

        for d in directions_to_avoid {
            let w = weights.entry(d).or_insert(0);
            *w += weights::AVOID_COLLIDING;
        }

        weights
    }
}
