use pathfinding::astar;
use std::borrow::Borrow;
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
        self.position.hash(state);
    }
}

impl PathNode {
    fn heuristic_to(&self, other: &PathNode) -> usize {
        self.position.manhattan_distance_to(&other.position) as usize
    }

    fn neighbours(&self) -> Vec<(PathNode, usize)> {
        self.map_information
            .neighbouring_turning_points(&self.position)
            .iter()
            .map(|p| (p, self.position.manhattan_distance_to(&p) as usize))
            .map(|(p, d)| (PathNode {
                position: p.clone(),
                map_information: self.map_information.clone(),
                current_map_state: self.current_map_state.clone(),
            }, d))
            .collect()
    }
}

pub fn get_shortest(from: &Position, to: &Position, map_information: &Rc<game::MapInformation>, map_state: &Rc<game::Map>) -> Option<(Vec<Position>, usize)> {
    println!("\n\nPathfinding from {} to {}", from, to);
    if *from == *to {
        return None; // We're already here
    }

    let source_intersection = map_information.neighbouring_turning_points(&from)
        .iter()
        // Get the one with the lowest manhattan distance to where we want to go
        .min_by(|p1, p2| {
            let d1 = p1.manhattan_distance_to(&to);
            let d2 = p2.manhattan_distance_to(&to);
            d1.cmp(&d2)
        })
        .and_then(|p| Some(p.clone()))
        .expect("Something went very wrong, I could not find an intersection to start pathfinding from..");

    let target_intersection = map_information.neighbouring_turning_points(&to)
        .iter()
        // Get the one with the lowest manhattan distance to where we want to go
        .min_by(|p1, p2| {
            let d1 = p1.manhattan_distance_to(&to);
            let d2 = p2.manhattan_distance_to(&to);
            d1.cmp(&d2)
        })
        .and_then(|p| Some(p.clone()))
        .expect("Something went very wrong, I could not find an intersection to end pathfinding at..");

    println!("Starting at intersection {} and going to {}", source_intersection, target_intersection);

    let source_node = PathNode {
        position: source_intersection,
        map_information: map_information.clone(),
        current_map_state: map_state.clone(),
    };

    let target_node = PathNode {
        position: target_intersection,
        map_information: map_information.clone(),
        current_map_state: map_state.clone(),
    };

    let path = astar(&source_node, |node| node.neighbours(), |node| node.heuristic_to(&target_node), |node| *node == target_node);

    if let Some(x) = path {
        let mut sequence: Vec<Position> = x.0
            .into_iter()
            .map(|node| node.position)
            .collect();

        // // Append the remaining movements needed from the last node (intersection) to our target tile
        // if !map_information.turning_points().contains(sequence.last().unwrap()) {
        //     let mut remaining = sequence.last().unwrap().direct_moves_to::<game::MapInformation>(&to, map_information.borrow());
        //     sequence.append(&mut remaining);
        // }

        // "Reverse" order is easier, we pop from the end while walking
        sequence.reverse();

        // // And now we can append the positions we need to visit to go from out tile to the first node
        // if !map_information.turning_points().contains(sequence.last().unwrap()) {
        //     let mut initial = from.direct_moves_to::<game::MapInformation>(&sequence.last().unwrap(), map_information.borrow());
        //     sequence.append(&mut initial);
        // }

        if *sequence.last().unwrap() == *from {
            sequence.pop();
        }

        // pathfinding crate: "The returned path comprises both the start and end node."
        // We don't need the start position
        // sequence.pop();

        // println!("Walking from {} to {}, a path of {} steps", sequence[0], sequence.last().unwrap(), sequence.len());
        // println!("Sequence:");
        // for p in &sequence {
        //     println!("\t{}", p);
        // }

        return Some((sequence, x.1));
    }

    None
}

#[cfg(all(test, feature = "benchmarking"))]
mod tests {
    extern crate test;

    use super::*;
    use self::test::Bencher;
    use serde_json;
    use game::{Map, MapInformation};

    const DEFAULT_MAP: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|....|................|....|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|....|.||....||....||.|....|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||--|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||--|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|....|.......||.......|....|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":238,"width":28}"#;

    #[bench]
    fn bench_get_shortest(b: &mut Bencher) {
        let map: Rc<Map> = Rc::new(serde_json::from_str(DEFAULT_MAP).unwrap());
        let info = Rc::new(MapInformation::from_map(&map));

        // Bench from a bit into the dead end in the bottom left of the map
        let origin = PathNode {
            position: Position {
                x: 3,
                y: 20,
            },
            map_information: info.clone(),
            current_map_state: map.clone(),
        };

        // To the top right of the map
        let destination = PathNode {
            position: Position {
                x: 18,
                y: 1,
            },
            map_information: info.clone(),
            current_map_state: map.clone(),
        };

        // Results from my i7 6700HQ, latest result at the top
        // (4c8b02c) 86,150 ns/iter (+/- 11,872) == 0.08615, a 66.69% improvement
        // (7175dc5) 258,640 ns/iter (+/- 19,377) == 0.25864 ms
        b.iter(|| {
            assert!(get_shortest(&origin, &destination).is_some());
        })
    }
}
