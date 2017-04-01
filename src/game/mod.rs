use std::collections::HashSet;

use common;
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

    pub fn tile_at(&self, position: &common::Position) -> TileType {
        self.tiles[(self.width * position.y + position.x) as usize]
    }

    pub fn neighbours(&self, position: &common::Position) -> Vec<(common::Direction, TileType)> {
        vec![
            (common::Direction::Left, self.tile_at(&position.adjacent(self, &common::Direction::Left))),
            (common::Direction::Right, self.tile_at(&position.adjacent(self, &common::Direction::Right))),
            (common::Direction::Up, self.tile_at(&position.adjacent(self, &common::Direction::Up))),
            (common::Direction::Down, self.tile_at(&position.adjacent(self, &common::Direction::Down))),
        ]
    }

    pub fn points_in_path(&self, path: &Vec<common::Position>) -> usize {
        path.iter()
            .filter(|pos| self.tile_at(pos).is_pellet())
            .count()
    }

    pub fn pellets(&self) -> HashSet<common::Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|&(_, tile)| tile.is_pellet())
            .map(|(i, &_)| i)
            .map(|i| self.index_to_position(i))
            .collect()
    }

    pub fn super_pellets(&self) -> HashSet<common::Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|&(_, tile)| tile.is_super_pellet())
            .map(|(i, &_)| i)
            .map(|i| self.index_to_position(i))
            .collect()
    }

    fn index_to_position(&self, index: usize) -> common::Position {
        let y = index as u32 / self.width;
        common::Position {
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
    turning_points: HashSet<common::Position>,

    intersections: HashSet<common::Position>,
    corners: HashSet<common::Position>,
    dead_ends: HashSet<common::Position>,
    tunnels: HashSet<common::Position>,

    walkable_positions: HashSet<common::Position>,
}

impl MapInformation {
    pub fn from_map(map: &Map) -> MapInformation {
        let mut map_information = MapInformation::default();

        // Classify what is walkable
        for y in 0..map.height() {
            for x in 0..map.width() {
                let pos = common::Position::new(x, y);
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
            let walkable_neighbours: Vec<(common::Direction, TileType)> = position.neighbours(map)
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

        map_information
    }

    pub fn turning_points(&self) -> &HashSet<common::Position> {
        // Return intersecions, corners and dead ends,
        // i.e. all positions where it would be sane to turn
        &self.turning_points
    }

    pub fn is_turning_point(&self, position: &common::Position) -> bool {
        self.turning_points.contains(position)
    }

    pub fn closest_turning_points<T: HasDimensions>(&self, limits: &T, position: &common::Position) -> HashSet<common::Position> {
        // Return the closest (1-4) intersections
        common::Direction::hash_set_all()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std;

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
        assert!(info.intersections.contains(&common::Position::new(3, 3)));
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
        assert!(info.intersections.contains(&common::Position::new(3, 2)));
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
        assert!(info.corners.contains(&common::Position::new(2, 2)));
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
        assert!(pellets.contains(&common::Position::new(1, 2)));
        assert!(pellets.contains(&common::Position::new(3, 3)));
    }
}
