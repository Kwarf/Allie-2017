use std::collections::{HashSet, VecDeque};

use common::{Direction, Position};
use protocol::json;
use traits::HasDimensions;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    Door,
    Pellet,
    SuperPellet,
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        match *self {
            TileType::Floor | TileType::Door | TileType::Pellet | TileType::SuperPellet => true,
            _ => false,
        }
    }

    pub fn is_pellet(&self) -> bool {
        match *self {
            TileType::Pellet | TileType::SuperPellet => true,
            _ => false,
        }
    }

    pub fn is_super_pellet(&self) -> bool {
        match *self {
            TileType::SuperPellet => true,
            _ => false,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Map {
    #[serde(rename = "content", deserialize_with = "json::deserialize_map_content")]
    tiles: Vec<TileType>,
    width: u32,
}

impl Map {
    pub fn tiles(&self) -> &[TileType] {
        self.tiles.as_slice()
    }

    pub fn tile_at(&self, position: &Position) -> TileType {
        self.tiles[(self.width * position.y + position.x) as usize]
    }

    pub fn neighbours(&self, position: &Position) -> Vec<(Direction, TileType)> {
        vec![
            (Direction::Left, self.tile_at(&position.adjacent(self, &Direction::Left))),
            (Direction::Right, self.tile_at(&position.adjacent(self, &Direction::Right))),
            (Direction::Up, self.tile_at(&position.adjacent(self, &Direction::Up))),
            (Direction::Down, self.tile_at(&position.adjacent(self, &Direction::Down))),
        ]
    }

    pub fn points_in_path(&self, path: &Vec<Position>) -> usize {
        path.iter()
            .filter(|pos| self.tile_at(pos).is_pellet())
            .count()
    }

    pub fn pellets(&self) -> HashSet<Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|&(_, tile)| tile.is_pellet())
            .map(|(i, &_)| i)
            .map(|i| self.index_to_position(i))
            .collect()
    }

    pub fn super_pellets(&self) -> HashSet<Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|&(_, tile)| tile.is_super_pellet())
            .map(|(i, &_)| i)
            .map(|i| self.index_to_position(i))
            .collect()
    }

    fn index_to_position(&self, index: usize) -> Position {
        let y = index as u32 / self.width;
        Position {
            x: index as u32 - self.width * y,
            y: y,
        }
    }
}

impl HasDimensions for Map {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.tiles.len() as u32 / self.width
    }
}

#[derive(Default)]
pub struct MapInformation {
    // This is essentially a combination of the three below, for convenience
    turning_points: HashSet<Position>,

    intersections: HashSet<Position>,
    corners: HashSet<Position>,
    dead_ends: HashSet<Position>,
    tunnels: HashSet<Position>,

    walkable_positions: HashSet<Position>,
}

impl MapInformation {
    pub fn from_map(map: &Map) -> MapInformation {
        let mut map_information = MapInformation::default();

        // Classify what is walkable
        for y in 0..map.height() {
            for x in 0..map.width() {
                let pos = Position::new(x, y);
                if map.tile_at(&pos).is_walkable() {
                    if pos.x == 0 || pos.x == map.width() - 1 || pos.y == 0 || pos.y == map.height() - 1 {
                        map_information.tunnels.insert(pos.clone());
                    }

                    map_information.walkable_positions.insert(pos);
                }
            }
        }

        // Find any intersections
        for position in &map_information.walkable_positions {
            let walkable_neighbours: Vec<(Direction, TileType)> = position.neighbours(map)
                .into_iter()
                .filter(|x| map_information.walkable_positions.contains(x))
                .map(|x| (position.direction_to(map, &x).unwrap(), map.tile_at(&x)))
                .collect();

            if walkable_neighbours.len() > 2 {
                map_information.intersections.insert(position.clone());
                map_information.turning_points.insert(position.clone());
            }
            else if walkable_neighbours.len() == 2
                && !walkable_neighbours[0].0.is_opposite_to(&walkable_neighbours[1].0)
                && !map_information.tunnels.contains(position) {
                map_information.corners.insert(position.clone());
                map_information.turning_points.insert(position.clone());
            }
            else if walkable_neighbours.len() == 1 {
                map_information.dead_ends.insert(position.clone());
                map_information.turning_points.insert(position.clone());
            }
        }

        // Classify tiles in dead ends
        let mut candidates = VecDeque::new();
        candidates.append(&mut map_information.dead_ends.clone().into_iter().collect());
        while let Some(c) = candidates.pop_front() {
            let dead_neighbours: Vec<Position> = c.neighbours(map)
                .into_iter()
                .filter(|p| {
                    map.tile_at(&p).is_walkable() &&
                    !map_information.dead_ends.contains(p) &&
                    p.neighbours(map).iter().filter(|pn| map.tile_at(&pn).is_walkable() && !map_information.dead_ends.contains(pn)).count() < 2
                })
                .collect();

            for n in dead_neighbours {
                map_information.dead_ends.insert(n.clone());
                candidates.push_back(n);
            }
        }

        map_information
    }

    pub fn intersections(&self) -> &HashSet<Position> {
        // Return intersecions, tiles with >2 directions to go
        // i.e. all tiles where a decision on where to go is needed
        &self.intersections
    }

    pub fn walkable_positions(&self) -> &HashSet<Position> {
        &self.walkable_positions
    }

    pub fn closest_turning_points<T: HasDimensions>(&self, limits: &T, position: &Position) -> HashSet<Position> {
        // Return the closest (1-4) intersections
        Direction::hash_set_all()
            .iter()
            .map(|d| {
                let mut p = position.clone();
                loop {
                    p = p.adjacent(limits, &d);
                    if self.turning_points.contains(&p) {
                        return Some(p);
                    }
                    if !self.walkable_positions.contains(&p) {
                        return None;
                    }
                }
            })
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn dead_ends(&self) -> &HashSet<Position> {
        &self.dead_ends
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std;

    const DEFAULT: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|....|................|....|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|....|.||....||....||.|....|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||--|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||--|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|....|.......||.......|....|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":238,"width":28}"#;
    const MSPACMAN1: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|......||..........||......|","|o||||.||.||||||||.||.||||o|","|.||||.||.||||||||.||.||||.|","|..........................|","|||.||.|||||.||.|||||.||.|||","__|.||.|||||.||.|||||.||.|__","|||.||.|||||.||.|||||.||.|||","___.||.......||.......||.___","|||.|||||_||||||||_|||||.|||","__|.|||||_||||||||_|||||.|__","__|.____________________.|__","__|.|||||_|||--|||_|||||.|__","__|.|||||_|______|_|||||.|__","__|.||____|______|____||.|__","__|.||_||_|______|_||_||.|__","|||.||_||_|||--|||_||_||.|||","___.___||__________||___.___","|||.||||||||_||_||||||||.|||","__|.||||||||_||_||||||||.|__","__|.......___||___.......|__","__|.|||||.||||||||.|||||.|__","|||.|||||.||||||||.|||||.|||","|............__............|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|.||||.||....||....||.||||.|","|o||||.||.||||||||.||.||||o|","|.||||.||.||||||||.||.||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":220,"width":28}"#;
    const MSPACMAN2: &'static str = r#"{"content":["||||||||||||||||||||||||||||","_______||..........||_______","||||||_||.||||||||.||_||||||","||||||_||.||||||||.||_||||||","|o...........||...........o|","|.|||||||.||.||.||.|||||||.|","|.|||||||.||.||.||.|||||||.|","|.||......||.||.||......||.|","|.||.||||_||....||_||||.||.|","|.||.||||_||||||||_||||.||.|","|......||_||||||||_||......|","||||||.||__________||.||||||","||||||.||_|||--|||_||.||||||","|......||_|______|_||......|","|.||||.||_|______|_||.||||.|","|.||||.___|______|___.||||.|","|...||.||_|||--|||_||.||...|","|||.||.||__________||.||.|||","__|.||.||||_||||_||||.||.|__","__|.||.||||_||||_||||.||.|__","__|.........||||.........|__","__|.|||||||.||||.|||||||.|__","|||.|||||||.||||.|||||||.|||","___....||...____...||....___","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|o..||.......||.......||..o|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":240,"width":28}"#;
    const MSPACMAN3: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|.........||....||.........|","|.|||||||.||.||.||.|||||||.|","|o|||||||.||.||.||.|||||||o|","|.||.........||.........||.|","|.||.||.||||.||.||||.||.||.|","|....||.||||.||.||||.||....|","||||.||.||||.||.||||.||.||||","||||.||..............||.||||","_....||||_||||||||_||||...._","|.||_||||_||||||||_||||_||.|","|.||____________________||.|","|.||||_||_|||--|||_||_||||.|","|.||||_||_|______|_||_||||.|","|._____||_|______|_||_____.|","|.||_||||_|______|_||||_||.|","|.||_||||_|||--|||_||||_||.|","|.||____________________||.|","|.||||_|||||_||_|||||_||||.|","|.||||_|||||_||_|||||_||||.|","|......||....||....||......|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|o..||.......__.......||..o|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|......||....||....||......|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|......||..........||......|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":238,"width":28}"#;
    const MSPACMAN4: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|..........................|","|.||.||||.||||||||.||||.||.|","|o||.||||.||||||||.||||.||o|","|.||.||||.||....||.||||.||.|","|.||......||.||.||......||.|","|.||||.||.||.||.||.||.||||.|","|.||||.||.||.||.||.||.||||.|","|......||....||....||......|","|||.||||||||_||_||||||||.|||","__|.||||||||_||_||||||||.|__","__|....||__________||....|__","|||_||.||_|||--|||_||.||_|||","____||.||_|______|_||.||____","||||||.___|______|___.||||||","||||||.||_|______|_||.||||||","____||.||_|||--|||_||.||____","|||_||.||__________||.||_|||","__|....|||||_||_|||||....|__","__|.||.|||||_||_|||||.||.|__","__|.||....___||___....||.|__","__|.|||||.||_||_||.|||||.|__","|||.|||||.||_||_||.|||||.|||","|.........||____||.........|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|.||...||..........||...||.|","|o||.|||||||.||.|||||||.||o|","|.||.|||||||.||.|||||||.||.|","|............||............|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":234,"width":28}"#;
    const PACMAN: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|..........................|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|......||....||....||......|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||--|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||--|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":240,"width":28}"#;

    #[test]
    fn should_be_able_to_determine_walkable_tiles() {
        // Walls are not walkable..
        assert!(!TileType::Wall.is_walkable());
        // Everything else is
        assert!(TileType::Floor.is_walkable());
        assert!(TileType::Door.is_walkable());
        assert!(TileType::Pellet.is_walkable());
        assert!(TileType::SuperPellet.is_walkable());
    }

    #[test]
    fn should_have_optimal_tile_enum_size() {
        // Make sure that tiles are 1B, as we store and index lots of them
        assert_eq!(1, std::mem::size_of::<TileType>());
    }

    #[test]
    fn can_find_simple_x_intersections() {
        const SIMPLE_INTERSECTION: &'static str = r#"
{
    "content": [
        "|||||||",
        "|||_|||",
        "|||_|||",
        "|_____|",
        "|||_|||",
        "|||_|||",
        "|||||||"
    ],
    "height": 7,
    "pelletsleft": 0,
    "width": 7
}"#;
        let map: Map = serde_json::from_str(SIMPLE_INTERSECTION).unwrap();
        let info = MapInformation::from_map(&map);
        assert_eq!(5, info.turning_points.len());
        assert_eq!(1, info.intersections.len());
        assert_eq!(0, info.corners.len());
        assert!(info.intersections.contains(&Position::new(3, 3)));
    }

    #[test]
    fn can_find_simple_three_way_intersections() {
        const THREE_WAY_INTERSECTION: &'static str = r#"
{
    "content": [
        "|||||",
        "|||_|",
        "|___|",
        "|||_|",
        "|||_|",
        "|||||"
    ],
    "height": 6,
    "pelletsleft": 0,
    "width": 5
}"#;
        let map: Map = serde_json::from_str(THREE_WAY_INTERSECTION).unwrap();
        let info = MapInformation::from_map(&map);
        assert_eq!(4, info.turning_points.len());
        assert_eq!(1, info.intersections.len());
        assert_eq!(0, info.corners.len());
        assert!(info.intersections.contains(&Position::new(3, 2)));
    }

    #[test]
    fn can_find_simple_turn() {
        const TURN: &'static str = r#"
{
    "content": [
        "||||",
        "||_|",
        "|__|",
        "||||"
    ],
    "height": 4,
    "pelletsleft": 0,
    "width": 4
}"#;
        let map: Map = serde_json::from_str(TURN).unwrap();
        let info = MapInformation::from_map(&map);
        assert_eq!(3, info.turning_points.len());
        assert_eq!(0, info.intersections.len());
        assert_eq!(1, info.corners.len());
        assert!(info.corners.contains(&Position::new(2, 2)));
    }

    #[test]
    fn should_not_find_corners_in_straight_paths() {
        const STRAIGHT: &'static str = r#"
{
    "content": [
        "||||",
        "||_|",
        "||_|",
        "||_|",
        "||_|",
        "||||"
    ],
    "height": 6,
    "pelletsleft": 0,
    "width": 4
}"#;
        let map: Map = serde_json::from_str(STRAIGHT).unwrap();
        let info = MapInformation::from_map(&map);
        assert_eq!(2, info.turning_points.len());
        assert_eq!(0, info.intersections.len());
        assert_eq!(0, info.corners.len());
    }

    #[test]
    fn tunnels_are_not_turning_points() {
        const TESTMAP: &'static str = r#"
{
    "content": [
        "|||||||",
        "|||_|||",
        "_______",
        "|||_|||",
        "|||||||"
    ],
    "height": 5,
    "pelletsleft": 0,
    "width": 7
}"#;
        let map: Map = serde_json::from_str(TESTMAP).unwrap();
        let info = MapInformation::from_map(&map);
        assert_eq!(2, info.tunnels.len());
        assert_eq!(2, info.dead_ends.len());
        assert_eq!(1, info.intersections.len());
        assert_eq!(0, info.corners.len());
        assert_eq!(3, info.turning_points.len());
    }

    #[test]
    fn can_find_pellet() {
        const THREE_WAY_INTERSECTION: &'static str = r#"
{
    "content": [
        "|||||",
        "|||_|",
        "|o__|",
        "|||o|",
        "|||_|",
        "|||||"
    ],
    "height": 6,
    "pelletsleft": 0,
    "width": 5
}"#;
        let map: Map = serde_json::from_str(THREE_WAY_INTERSECTION).unwrap();
        let pellets = map.pellets();
        assert_eq!(2, pellets.len());
        assert!(pellets.contains(&Position::new(1, 2)));
        assert!(pellets.contains(&Position::new(3, 3)));
    }

    #[test]
    fn can_classify_dead_ends() {
        // 18 * 2 + 11 * 2 == 58 number of tiles that should be classified as belonging to dead ends
        let info = MapInformation::from_map(&serde_json::from_str::<Map>(DEFAULT).unwrap());
        assert_eq!(58, info.dead_ends().len());

        // 4 tiles (unreachable)
        let info = MapInformation::from_map(&serde_json::from_str::<Map>(MSPACMAN1).unwrap());
        assert_eq!(4, info.dead_ends().len());

        // The remaining maps have no dead ends
        let info = MapInformation::from_map(&serde_json::from_str::<Map>(MSPACMAN2).unwrap());
        assert_eq!(0, info.dead_ends().len());
        let info = MapInformation::from_map(&serde_json::from_str::<Map>(MSPACMAN3).unwrap());
        assert_eq!(0, info.dead_ends().len());
        let info = MapInformation::from_map(&serde_json::from_str::<Map>(MSPACMAN4).unwrap());
        assert_eq!(0, info.dead_ends().len());
        let info = MapInformation::from_map(&serde_json::from_str::<Map>(PACMAN).unwrap());
        assert_eq!(0, info.dead_ends().len());
    }
}

#[cfg(all(test, feature = "benchmarking"))]
mod benchmarks {
    extern crate test;

    use super::*;
    use self::test::Bencher;
    use serde_json;

    const DEFAULT: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|....|................|....|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|....|.||....||....||.|....|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||--|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||--|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|....|.......||.......|....|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":238,"width":28}"#;

    #[bench]
    fn bench_info_from_map(b: &mut Bencher) {
        b.iter(|| {
            test::black_box(serde_json::from_str::<Map>(DEFAULT).unwrap());
        })
    }

    #[bench]
    fn bench_hashset_lookup(b: &mut Bencher) {
        let map: Map = serde_json::from_str(DEFAULT).unwrap();
        let info = MapInformation::from_map(&map);

        let walkable = Position::new(26, 21);
        let wall = Position::new(7, 22);

        b.iter(|| {
            test::black_box(info.walkable_positions().contains(&walkable));
            test::black_box(info.walkable_positions().contains(&wall));
        })
    }

    #[bench]
    fn bench_array_lookup(b: &mut Bencher) {
        let map: Map = serde_json::from_str(DEFAULT).unwrap();

        let walkable = Position::new(26, 21);
        let wall = Position::new(7, 22);

        b.iter(|| {
            test::black_box(map.tile_at(&walkable).is_walkable());
            test::black_box(map.tile_at(&wall).is_walkable());
        })
    }
}
