use std::cell::RefCell;
use std::collections::HashMap;

mod pathfinder;
mod strategies;

use ai::strategies::{Strategy, weights};
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
    pub fn from_game_state(state: &protocol::GameState) -> Bot {
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

        // println!("");
        let decision = self.strategies
            .iter()
            .map(|x| x.borrow_mut().action(&self, &state))
            // .map(|x| {
            //     let weights = x.borrow_mut().action(&self, &state);
            //     println!("{:?}: {:?}", x.borrow().description(), weights);
            //     weights
            // })
            .fold(self.directions_with_default_weights(&state.map, &self.map_information, &state.me.position()), |mut acc, x| {
                for (d, f) in x {
                    let w = acc.entry(d).or_insert(0);
                    *w += f;
                }
                acc
            })
            .into_iter()
            // .map(|x| {
            //     println!("{} {}", x.0, x.1);
            //     x
            // })
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

    fn directions_with_default_weights(&self, map: &game::Map, map_information: &game::MapInformation, my_pos: &Position) -> HashMap<Direction, i32> {
        Direction::hash_set_all()
            .into_iter()
            .map(|d| (d.clone(), if map.tile_at(&my_pos.adjacent(map, &d)).is_walkable() { if map_information.is_dead_end(&my_pos.adjacent(map, &d))  { weights::DEAD_END_PENALTY } else { 0 } } else { weights::WALL_PENALTY }))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    use ai::Bot;
    use game::Map;
    use protocol::Message;
    use traits::HasPosition;

    const DEFAULT_MAP: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|....|................|....|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|....|.||....||....||.|....|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||--|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||--|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|....|.......||.......|....|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":238,"width":28}"#;

    #[test]
    fn should_have_lower_weights_for_walls() {
        const STATE: &'static str = r#"{"gamestate":{"map":{"content":["||||||||||||||||||||||||||||","|____________||____________|","|_||||_|||||_||_|||||_||||_|","|_||||_|||||_||_|||||_||||_|","|.||||_|||||_||_|||||_||||.|","|....|________________|....|","|.||||_||_||||||||_||_||||.|","|.||||_||_||||||||_||_||||.|","|....|_||____||____||_|....|","||||||_|||||_||_|||||_||||||","_____|_|||||_||_|||||_|_____","_____|_||__________||_|_____","_____|_||_|||--|||_||_|_____","||||||_||_|______|_||_||||||","__________|______|__________","||||||_||_|______|_||_||||||","_____|_||_|||--|||_||_|_____","_____|_||__________||_|_____","_____|_||_||||||||_||_|_____","||||||_||_||||||||_||_||||||","|....|_______||_______|....|","|.||||_|||||_||_|||||_||||.|","|.||||_|||||_||_|||||_||||.|","|o..||________________||..o|","|||.||_||_||||||||_||_||.|||","|||.||_||_||||||||_||_||.|||","|______||____||____||______|","|_||||||||||_||_||||||||||_|","|_||||||||||_||_||||||||||_|","|__________________________|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":42,"width":28},"others":[{"id":0,"isdangerous":true,"score":59,"x":21,"y":4}],"you":{"id":1,"isdangerous":false,"score":153,"x":20,"y":1}},"messagetype":"stateupdate"}"#;
        let message = protocol::Message::from_str(STATE).unwrap();

        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();
        let map_information = game::MapInformation::from_map(&map);

        match message {
            Message::Update { state } => {
                let bot = Bot::from_game_state(&state);
                let weights = bot.directions_with_default_weights(&map, &map_information, &state.me.position());

                assert_eq!(4, weights.len());
                assert_eq!(0, *weights.get(&Direction::Left).unwrap());
                assert_eq!(0, *weights.get(&Direction::Right).unwrap());
                assert_eq!(weights::WALL_PENALTY, *weights.get(&Direction::Up).unwrap());
                assert_eq!(weights::WALL_PENALTY, *weights.get(&Direction::Down).unwrap());
            },
            _ => panic!()
        }
    }

    #[test]
    fn should_not_die_at_dead_end_entrance() {
        const STATE: &'static str = r#"{"gamestate":{"map":{"content":["||||||||||||||||||||||||||||","|____________||____________|","|_||||_|||||_||_|||||_||||_|","|_||||_|||||_||_|||||_||||_|","|.||||_|||||_||_|||||_||||.|","|....|________________|....|","|.||||_||_||||||||_||_||||.|","|.||||_||_||||||||_||_||||.|","|....|_||____||____||_|....|","||||||_|||||_||_|||||_||||||","_____|_|||||_||_|||||_|_____","_____|_||__________||_|_____","_____|_||_|||--|||_||_|_____","||||||_||_|______|_||_||||||","__________|______|__________","||||||_||_|______|_||_||||||","_____|_||_|||--|||_||_|_____","_____|_||__________||_|_____","_____|_||_||||||||_||_|_____","||||||_||_||||||||_||_||||||","|....|_______||_______|....|","|.||||_|||||_||_|||||_||||.|","|.||||_|||||_||_|||||_||||.|","|o..||________________||..o|","|||.||_||_||||||||_||_||.|||","|||.||_||_||||||||_||_||.|||","|______||____||____||______|","|_||||||||||_||_||||||||||_|","|_||||||||||_||_||||||||||_|","|__________________________|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":42,"width":28},"others":[{"id":0,"isdangerous":true,"score":59,"x":21,"y":4}],"you":{"id":1,"isdangerous":false,"score":153,"x":21,"y":1}},"messagetype":"stateupdate"}"#;
        let message = protocol::Message::from_str(STATE).unwrap();

        match message {
            Message::Update { state } => {
                let mut bot = Bot::from_game_state(&state);
                let action = bot.determine_action(state);
                assert_eq!(Direction::Left, action);
            },
            _ => panic!()
        }
    }

    #[test]
    fn should_not_die_stupidly() {
        const STATE: &'static str = r#"{"gamestate":{"map":{"content":["||||||||||||||||||||||||||||","|____________||____________|","|_||||_|||||_||_|||||_||||_|","|_||||_|||||_||_|||||_||||_|","|.||||_|||||_||_|||||_||||.|","|....|________________|....|","|.||||_||_||||||||_||_||||.|","|.||||_||_||||||||_||_||||.|","|....|_||____||____||_|....|","||||||_|||||_||_|||||_||||||","_____|_|||||_||_|||||_|_____","_____|_||__________||_|_____","_____|_||_|||--|||_||_|_____","||||||_||_|______|_||_||||||","__________|______|__________","||||||_||_|______|_||_||||||","_____|_||_|||--|||_||_|_____","_____|_||__________||_|_____","_____|_||_||||||||_||_|_____","||||||_||_||||||||_||_||||||","|....|_______||_______|....|","|.||||_|||||_||_|||||_||||.|","|.||||_|||||_||_|||||_||||.|","|o..||________________||..o|","|||.||_||_||||||||_||_||.|||","|||.||_||_||||||||_||_||.|||","|______||____||____||______|","|_||||||||||_||_||||||||||_|","|_||||||||||_||_||||||||||_|","|__________________________|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":42,"width":28},"others":[{"id":0,"isdangerous":true,"score":59,"x":21,"y":1}],"you":{"id":1,"isdangerous":false,"score":153,"x":20,"y":1}},"messagetype":"stateupdate"}"#;
        let message = protocol::Message::from_str(STATE).unwrap();

        match message {
            Message::Update { state } => {
                let mut bot = Bot::from_game_state(&state);
                let action = bot.determine_action(state);
                assert_eq!(Direction::Left, action);
            },
            _ => panic!()
        }
    }
}
