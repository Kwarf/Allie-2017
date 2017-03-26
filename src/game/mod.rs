use std;

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
}

// Should really not be Clone
#[derive(Clone, Debug, Deserialize)]
pub struct Map {
    #[serde(rename = "content", deserialize_with = "json::deserialize_map_content")]
    tiles: Vec<TileType>,
    width: u32,
}

impl Map {
    #[cfg(debug_assertions)]
    pub fn tilecount(&self) -> usize {
        self.tiles.len()
    }

    pub fn tile_at(&self, x: u32, y: u32) -> TileType {
        self.tiles[(self.width * y + x) as usize]
    }

    pub fn neighbours(&self, x: u32, y: u32) -> Vec<(common::Direction, TileType)> {
        let mut neighbours = Vec::new();
        if x > 0 {
            neighbours.push((common::Direction::Left, self.tile_at(x - 1, y)));
        }
        if x < self.width - 1 {
            neighbours.push((common::Direction::Right, self.tile_at(x + 1, y)));
        }
        if y > 0 {
            neighbours.push((common::Direction::Up, self.tile_at(x, y - 1)));
        }
        if y < self.height() - 1 {
            neighbours.push((common::Direction::Down, self.tile_at(x, y + 1)));
        }
        neighbours
    }

    pub fn points_in_path(&self, path: &Vec<common::Position>) -> usize {
        path.iter()
            .filter(|pos| self.tile_at(pos.x, pos.y).is_pellet())
            .count()
    }

    pub fn pellets(&self) -> Vec<common::Position> {
        self.tiles
            .iter()
            .enumerate()
            .filter(|&(i, tile)| tile.is_pellet())
            .map(|(i, &tile)| i)
            .map(|i| {
                let y = i as u32 / self.width;
                common::Position {
                    x: i as u32 - self.width * y,
                    y: y,
                }
            })
            .collect()
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
    intersections: Vec<common::Position>,
    corners: Vec<common::Position>,
    dead_ends: Vec<common::Position>,
}

impl MapInformation {
    pub fn from_map(map: &Map) -> MapInformation {
        let mut map_information = MapInformation::default();

        // Find any intersections
        for y in 0..map.height() {
            for x in 0..map.width() {
                if !map.tile_at(x, y).is_walkable() {
                    continue;
                }

                let walkable_neighbours: Vec<(common::Direction, TileType)> = map.neighbours(x, y)
                    .into_iter()
                    .filter(|x| x.1.is_walkable())
                    .collect();

                if walkable_neighbours.len() > 2 {
                    map_information.intersections.push(common::Position::new(x, y));
                }
                else if walkable_neighbours.len() == 2 && !walkable_neighbours[0].0.is_opposite_to(&walkable_neighbours[1].0) {
                    map_information.corners.push(common::Position::new(x, y));
                }
                else if walkable_neighbours.len() == 1 {
                    map_information.dead_ends.push(common::Position::new(x, y));
                }
            }
        }

        map_information
    }

    pub fn turning_points<'a>(&'a self) -> std::iter::Chain<std::iter::Chain<std::slice::Iter<'a, common::Position>, std::slice::Iter<'a, common::Position>>, std::slice::Iter<'a, common::Position>> {
        // Return a chain of intersecions, corners and dead ends,
        // i.e. all positions where it would be sane to turn
        self.intersections
            .iter()
            .chain(self.corners.iter())
            .chain(self.dead_ends.iter())
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
        assert_eq!(1, info.intersections.len());
        assert_eq!(0, info.corners.len());
        assert_eq!(3, info.intersections[0].x);
        assert_eq!(3, info.intersections[0].y);
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
        assert_eq!(1, info.intersections.len());
        assert_eq!(0, info.corners.len());
        assert_eq!(3, info.intersections[0].x);
        assert_eq!(2, info.intersections[0].y);
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
        assert_eq!(0, info.intersections.len());
        assert_eq!(1, info.corners.len());
        assert_eq!(2, info.corners[0].x);
        assert_eq!(2, info.corners[0].y);
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
        assert_eq!(0, info.intersections.len());
        assert_eq!(0, info.corners.len());
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
        assert_eq!(2, map.pellets().len());
        assert_eq!(common::Position { x: 1, y: 2 }, map.pellets()[0]);
        assert_eq!(common::Position { x: 3, y: 3 }, map.pellets()[1]);
    }
}
