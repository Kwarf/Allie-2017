use serde_json;

use protocol::Map;
use std;
use traits::HasDimensions;

pub struct Position {
    x: u32,
    y: u32,
}

impl Position {
    fn new(x: u32, y: u32) -> Position {
        Position {
            x: x,
            y: y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    Door,
    Pellet,
    SuperPellet,
}

impl TileType {
    fn is_walkable(&self) -> bool {
        match *self {
            TileType::Floor | TileType::Door | TileType::Pellet | TileType::SuperPellet => true,
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct MapInformation {
    intersections: Vec<Position>,
}

impl MapInformation {
    fn from_map(map: &Map) -> MapInformation {
        let mut map_information = MapInformation::default();

        // Find any intersections
        for y in 0..map.height() {
            for x in 0..map.width() {
                if !map.tile_at(x, y).is_walkable() {
                    continue;
                }

                let walkable_neighbour_count = map.neighbours(x, y)
                    .iter()
                    .filter(|x| x.1.is_walkable())
                    .count();
                if walkable_neighbour_count > 2 {
                    map_information.intersections.push(Position::new(x, y));
                }
            }
        }

        map_information
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(3, info.intersections[0].x);
        assert_eq!(2, info.intersections[0].y);
    }
}
