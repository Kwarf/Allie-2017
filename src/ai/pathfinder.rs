use pathfinding::{astar, bfs};
use std::borrow::Borrow;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use common::Position;
use game;
use traits::HasDimensions;

pub struct LocalPathGraph {
    nodes: HashMap<Position, TilePathInformation>,
}

#[derive(Clone)]
struct TilePathInformation {
    cost: usize,
    parent: Position,
}

impl LocalPathGraph {
    pub fn new(map: &game::Map) -> LocalPathGraph {
        LocalPathGraph {
            nodes: HashMap::with_capacity(map.tiles().iter().filter(|x| x.is_walkable()).count()),
        }
    }

    pub fn update_from_map(&mut self, map: &game::Map, my_position: &Position) {
        // Clear any existing information
        self.nodes.clear();

        let mut frontier = VecDeque::new();
        frontier.push_back((my_position.clone(), 0 as usize));

        while let Some((current, current_cost)) = frontier.pop_front() {
            for adjacent in current.neighbours(map) {
                if !map.tile_at(&adjacent).is_walkable() {
                    continue;
                }

                if !self.nodes.contains_key(&adjacent) && adjacent != *my_position {
                    let adjacent_information = TilePathInformation {
                        cost: current_cost + 1,
                        parent: current.clone(),
                    };
                    frontier.push_back((adjacent.clone(), current_cost + 1));
                    self.nodes.insert(adjacent, adjacent_information);
                }
            }
        }
    }

    pub fn cost_to(&self, position: &Position) -> Option<usize> {
        self.nodes
            .get(position)
            .and_then(|n| Some(n.cost))
    }

    pub fn path_to(&self, position: &Position) -> Option<Vec<Position>> {
        if !self.nodes.contains_key(position) {
            return None;
        }

        let mut path = Vec::new(); // TODO: Pre-allocate since we know the cost
        path.push(position.clone());
        while let Some(n) = self.nodes.get(&path.last().unwrap()) {
            path.push(n.parent.clone());
        }

        // Remove the last parent since that's our current position
        path.pop();

        Some(path)
    }
}

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

    fn walkable_neighbours<T: HasDimensions>(&self, limits: &T) -> Vec<PathNode> {
        self.current_map_state
            .neighbours(&self.position)
            .iter()
            .filter(|x| x.1.is_walkable())
            .map(|x| self.position.neighbour(limits, &x.0))
            .map(|p| PathNode {
                position: p,
                map_information: self.map_information.clone(),
                current_map_state: self.current_map_state.clone(),
            })
            .collect()
    }

    fn neighbours<T: HasDimensions>(&self, limits: &T) -> Vec<(PathNode, usize)> {
        self.walkable_neighbours(limits)
            .into_iter()
            .map(|n| (n, 1))
            .collect()
    }

    fn neighbours_by_points<T: HasDimensions>(&self, limits: &T) -> Vec<(PathNode, usize)> {
        self.walkable_neighbours(limits)
            .into_iter()
            .map(|n| {
                // Make it twice as expensive to walk on tiles that have no pellets
                let cost = if self.current_map_state.tile_at(&n.position).is_pellet() { 1 } else { 2 };
                (n, cost)
            })
            .collect()
    }
}

// This method uses breadth-first search to find the pellet closest to our position
pub fn find_closest_pellet(origin: &PathNode) -> Option<Vec<Position>> {
    let path = bfs(origin
        , |p| p.neighbours::<game::Map>(origin.current_map_state.borrow()).into_iter().map(|x| x.0) // Map away cost
        , |p| origin.current_map_state.tile_at(&p.position).is_pellet());

    if let Some(x) = path {
        let mut sequence: Vec<Position> = x
            .into_iter()
            .rev()
            .skip(1)
            .map(|node| node.position)
            .collect();

        return Some(sequence);
    }

    None
}

pub fn get_shortest(from: &PathNode, to: &PathNode) -> Option<Vec<Position>> {
    let path = astar(from, |p| p.neighbours::<game::Map>(from.current_map_state.borrow()), |p| p.heuristic_to(&to), |p| *p == *to);
    prepare_response(path)
}

pub fn get_by_points(from: &PathNode, to: &PathNode) -> Option<Vec<Position>> {
    let path = astar(from, |p| p.neighbours_by_points::<game::Map>(from.current_map_state.borrow()), |p| p.heuristic_to(&to), |p| *p == *to);
    prepare_response(path)
}

fn prepare_response(path: Option<(Vec<PathNode>, usize)>) -> Option<Vec<Position>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use game::{Map, MapInformation};

    const DEFAULT_MAP: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|....|................|....|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|....|.||....||....||.|....|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||--|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||--|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|....|.......||.......|....|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":238,"width":28}"#;

    #[test]
    fn bfs_path_graph_should_return_same_path_as_library() {
        let map: Rc<Map> = Rc::new(serde_json::from_str(DEFAULT_MAP).unwrap());
        let info = Rc::new(MapInformation::from_map(&map));

        let origin = PathNode {
            position: Position {
                x: 3,
                y: 20,
            },
            map_information: info.clone(),
            current_map_state: map.clone(),
        };

        let destination = PathNode {
            position: Position {
                x: 18,
                y: 1,
            },
            map_information: info.clone(),
            current_map_state: map.clone(),
        };

        assert_eq!(50, get_cost_from_bfs_graph(&map, &origin.position, &destination.position));

        let bfs_path = get_path_from_bfs_graph(&map, &origin.position, &destination.position);
        assert_eq!(50, bfs_path.len());

        let lib_path = get_path_from_astar_lib(&origin, &destination);
        assert_eq!(50, lib_path.len());

        assert_eq!(lib_path.as_slice(), bfs_path.as_slice());
    }

    fn get_cost_from_bfs_graph(map: &Map, from: &Position, to: &Position) -> usize {
        let mut graph = LocalPathGraph::new(&map);
        graph.update_from_map(&map, from);
        graph.cost_to(to).unwrap()
    }

    fn get_path_from_bfs_graph(map: &Map, from: &Position, to: &Position) -> Vec<Position> {
        let mut graph = LocalPathGraph::new(&map);
        graph.update_from_map(&map, from);
        graph.path_to(to).unwrap()
    }

    fn get_path_from_astar_lib(from: &PathNode, to: &PathNode) -> Vec<Position> {
        prepare_response(astar(from, |p| p.neighbours_by_points::<game::Map>(from.current_map_state.borrow()), |p| p.heuristic_to(&to), |p| *p == *to)).unwrap()
    }
}

#[cfg(all(test, feature = "benchmarking"))]
mod benchmarks {
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

    #[bench]
    fn bench_update_path_graph(b: &mut Bencher) {
        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();
        let mut graph = LocalPathGraph::new(&map);

        let origin = Position::new(3, 20);

        b.iter(|| {
            graph.update_from_map(&map, &origin);
        });
    }

    #[bench]
    fn bench_path_graph_path_query(b: &mut Bencher) {
        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();
        let mut graph = LocalPathGraph::new(&map);

        let origin = Position::new(3, 20);
        let destination = Position::new(18, 1);

        graph.update_from_map(&map, &origin);

        b.iter(|| {
            graph.path_to(&destination);
        });
    }
}
