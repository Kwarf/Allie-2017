use pathfinding::{astar, bfs};
use std::collections::{HashMap, VecDeque};

use common::Position;
use game;

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

// This method uses breadth-first search to find the pellet closest to our position
pub fn find_closest_pellet(map: &game::Map, origin: &Position) -> Option<Vec<Position>> {
    let path = bfs(origin
        , |p| p.neighbours(map)
        , |p| map.tile_at(&p).is_pellet());

    if let Some(x) = path {
        let mut sequence: Vec<Position> = x
            .into_iter()
            .rev()
            .skip(1)
            .collect();

        return Some(sequence);
    }

    None
}

pub fn get_shortest(map: &game::Map, from: &Position, to: &Position) -> Option<Vec<Position>> {
    let path = astar(from, |p| p.neighbours(map).into_iter().filter(|x| map.tile_at(x).is_walkable()).map(|x| (x, 1)), |p| p.manhattan_distance_to(&to, map) as usize, |p| *p == *to);
    prepare_response(path)
}

fn prepare_response(path: Option<(Vec<Position>, usize)>) -> Option<Vec<Position>> {
    if let Some(x) = path {
        let mut sequence: Vec<Position> = x.0
            .into_iter()
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
    use game::Map;

    const DEFAULT_MAP: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|....|................|....|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|....|.||....||....||.|....|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||--|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||--|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|....|.......||.......|....|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":238,"width":28}"#;

    #[test]
    fn bfs_path_graph_should_return_same_path_as_library() {
        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();

        let origin = Position::new(3, 20);
        let destination = Position::new(18, 1);

        assert_eq!(50, get_cost_from_bfs_graph(&map, &origin, &destination));

        let bfs_path = get_path_from_bfs_graph(&map, &origin, &destination);
        assert_eq!(50, bfs_path.len());

        let lib_path = get_path_from_astar_lib(&map, &origin, &destination);
        assert_eq!(50, lib_path.len());

        assert_eq!(lib_path.as_slice(), bfs_path.as_slice());
    }

    #[test]
    fn ensure_can_walk_wrapping() {
        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();

        let origin = Position::new(6, 13);
        let destination = Position::new(26, 14);

        assert_eq!(9, get_cost_from_bfs_graph(&map, &origin, &destination));

        let bfs_path = get_path_from_bfs_graph(&map, &origin, &destination);
        assert_eq!(9, bfs_path.len());

        let lib_path = get_path_from_astar_lib(&map, &origin, &destination);
        assert_eq!(9, lib_path.len());

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

    fn get_path_from_astar_lib(map: &Map, from: &Position, to: &Position) -> Vec<Position> {
        prepare_response(astar(from, |p| p.neighbours(map).into_iter().filter(|x| map.tile_at(x).is_walkable()).map(|x| (x, 1)), |p| p.manhattan_distance_to(&to, map) as usize, |p| *p == *to)).unwrap()
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
    fn bench_lib_astar(b: &mut Bencher) {
        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();

        let origin = Position::new(3, 20);
        let destination = Position::new(18, 1);

        // Results from my i7 6700HQ, latest result at the top
        // (63c250b) 52,840 ns/iter (+/- 10,839)
        // (4c8b02c) 86,150 ns/iter (+/- 11,872) == 0.08615, a 66.69% improvement
        // (7175dc5) 258,640 ns/iter (+/- 19,377) == 0.25864 ms
        b.iter(|| {
            astar(&origin, |p| p.neighbours(&map).into_iter().filter(|x| map.tile_at(x).is_walkable()).map(|x| (x, 1)), |p| p.manhattan_distance_to(&destination) as usize, |p| *p == destination)
        })
    }

    #[bench]
    fn bench_lib_bfs(b: &mut Bencher) {
        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();

        let origin = Position::new(3, 20);
        let destination = Position::new(18, 1);

        // (63c250b) 52,007 ns/iter (+/- 5,858)
        // (e4ab2b1) 106,758 ns/iter (+/- 15,305), only slightly slower than A* on this test case
        b.iter(|| {
            bfs(&origin, |p| p.neighbours(&map).into_iter().into_iter().filter(|x| map.tile_at(x).is_walkable()), |p| *p == destination)
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
