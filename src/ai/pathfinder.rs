use pathfinding::astar;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use common::{Direction, Position};
use game;

#[derive(Clone)]
pub struct PathNode {
    pub position: Position,

    // Is Rc really needed here? cba to make it work without it atm
    pub map_information: Rc<game::MapInformation>,
    pub current_map_state: Rc<game::Map>,
}

impl Eq for PathNode {}
impl PartialEq for PathNode {
    fn eq(&self, other: &PathNode) -> bool {
        self.position == other.position
    }
}

impl Hash for PathNode {
    fn hash<H: Hasher>(&self, state: &mut H) {

    }
}

impl PathNode {
    fn heuristic_to(&self, other: &PathNode) -> usize {
        self.position.manhattan_distance_to(&other.position) as usize
    }

    fn neighbours(&self) -> Vec<(PathNode, usize)> {
        self.current_map_state
            .neighbours(self.position.x, self.position.y)
            .iter()
            .filter(|x| x.1.is_walkable())
            .map(|x| {
                match x.0 {
                    Direction::Up => (0i32, -1i32),
                    Direction::Down => (0, 1),
                    Direction::Left => (-1, 0),
                    Direction::Right => (1, 0),
                }
            })
            .map(|p| Position {
                // Again with the typecasts.. Everything should have been i32
                x: (self.position.x as i32 + p.0) as u32,
                y: (self.position.y as i32 + p.1) as u32,
            })
            .map(|p| PathNode {
                position: p,
                map_information: self.map_information.clone(),
                current_map_state: self.current_map_state.clone(),
            })
            .map(|n| (n, 1))
            .collect()
    }
}

pub fn get_shortest(from: &PathNode, to: &PathNode) -> Option<Vec<Position>> {
    let path = astar(from, |p| p.neighbours(), |p| p.heuristic_to(&to), |p| *p == *to);

    if let Some(x) = path {
        let mut sequence: Vec<Position> = x.0
            .into_iter()
            .map(|node| node.position)
            .collect();

        // "Reverse" order is easier, we pop from the end while walking
        sequence.reverse();

        // pathfinding crate: "The returned path comprises both the start and end node."
        // We don't need the start position
        sequence.pop();

        return Some(sequence);
    }

    None
}