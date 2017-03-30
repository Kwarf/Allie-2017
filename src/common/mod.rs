use std::cmp;
use std::collections::HashSet;
use std::fmt;

pub mod rules;

use traits::HasDimensions;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Direction::Up => "UP",
            Direction::Down => "DOWN",
            Direction::Left => "LEFT",
            Direction::Right => "RIGHT",
        })
    }
}

impl Direction {
    pub fn is_opposite_to(&self, other: &Direction) -> bool {
        *self == Direction::Up && *other == Direction::Down ||
        *self == Direction::Down && *other == Direction::Up ||
        *self == Direction::Left && *other == Direction::Right ||
        *self == Direction::Right && *other == Direction::Left
    }

    pub fn hash_set_all() -> HashSet<Direction> {
        let mut set = HashSet::with_capacity(4);
        set.insert(Direction::Up);
        set.insert(Direction::Down);
        set.insert(Direction::Left);
        set.insert(Direction::Right);
        set
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Position {
        Position {
            x: x,
            y: y,
        }
    }

    pub fn manhattan_distance_to<T: HasDimensions>(&self, other: &Position, limits: &T) -> u32 {
        let (x1, x2) = (self.x as i32, other.x as i32);
        let (y1, y2) = (self.y as i32, other.y as i32);

        // "Regular" manhattan distance
        // ((x1 - x2).abs() + (y1 - y2).abs()) as u32

        let (w, h) = (limits.width() as i32 + 1, limits.height() as i32 + 1);
        let n = limits.height() as i32 + 1;
        let m = limits.width() as i32 + 1;

        // Manhattan distance for wrapping grid
        (cmp::min((x1 - x2).abs(), n - 1 - (x1 - x2).abs()) + cmp::min((y1 - y2).abs(), m - 1 - (y1 - y2).abs())) as u32
    }

    pub fn direction_to(&self, other: &Position) -> Option<Direction> {
        if other.x < self.x {
            return Some(Direction::Left);
        }
        else if other.x > self.x {
            return Some(Direction::Right);
        }

        if other.y < self.y {
            return Some(Direction::Up);
        }
        else if other.y > self.y {
            return Some(Direction::Down);
        }

        None
    }

    // Returns adjacent position in provided direction, with limit wrapping
    // And yes, it's not very nice looking, but I think it works
    pub fn adjacent<T: HasDimensions>(&self, limits: &T, direction: &Direction) -> Position {
        let width = limits.width() - 1;
        let height = limits.height() - 1;
        match *direction {
            Direction::Up => Position {
                x: self.x,
                y: if self.y == 0 { height } else { self.y - 1 },
            },
            Direction::Down => Position {
                x: self.x,
                y: if self.y == height { 0 } else { self.y + 1 },
            },
            Direction::Left => Position {
                x: if self.x == 0 { width } else { self.x - 1 },
                y: self.y
            },
            Direction::Right => Position {
                x: if self.x == width { 0 } else { self.x + 1 },
                y: self.y,
            },
        }
    }

    pub fn neighbours<T: HasDimensions>(&self, limits: &T) -> Vec<Position> {
        vec![
            self.adjacent(limits, &Direction::Up),
            self.adjacent(limits, &Direction::Down),
            self.adjacent(limits, &Direction::Left),
            self.adjacent(limits, &Direction::Right),
        ]
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use game::Map;

    const DEFAULT_MAP: &'static str = r#"{"content":["||||||||||||||||||||||||||||","|............||............|","|.||||.|||||.||.|||||.||||.|","|o||||.|||||.||.|||||.||||o|","|.||||.|||||.||.|||||.||||.|","|....|................|....|","|.||||.||.||||||||.||.||||.|","|.||||.||.||||||||.||.||||.|","|....|.||....||....||.|....|","||||||.|||||_||_|||||.||||||","_____|.|||||_||_|||||.|_____","_____|.||__________||.|_____","_____|.||_|||--|||_||.|_____","||||||.||_|______|_||.||||||","______.___|______|___.______","||||||.||_|______|_||.||||||","_____|.||_|||--|||_||.|_____","_____|.||__________||.|_____","_____|.||_||||||||_||.|_____","||||||.||_||||||||_||.||||||","|....|.......||.......|....|","|.||||.|||||.||.|||||.||||.|","|.||||.|||||.||.|||||.||||.|","|o..||.......__.......||..o|","|||.||.||.||||||||.||.||.|||","|||.||.||.||||||||.||.||.|||","|......||....||....||......|","|.||||||||||.||.||||||||||.|","|.||||||||||.||.||||||||||.|","|..........................|","||||||||||||||||||||||||||||"],"height":31,"pelletsleft":238,"width":28}"#;

    #[test]
    fn can_get_adjacent_positions() {
        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();

        assert_eq!(Position::new(10, 9), Position::new(10, 10).adjacent(&map, &Direction::Up));
        assert_eq!(Position::new(10, 11), Position::new(10, 10).adjacent(&map, &Direction::Down));
        assert_eq!(Position::new(9, 10), Position::new(10, 10).adjacent(&map, &Direction::Left));
        assert_eq!(Position::new(11, 10), Position::new(10, 10).adjacent(&map, &Direction::Right));

        // Wrapping
        assert_eq!(Position::new(0, 30), Position::new(0, 0).adjacent(&map, &Direction::Up));
        assert_eq!(Position::new(27, 0), Position::new(0, 0).adjacent(&map, &Direction::Left));

        assert_eq!(Position::new(27, 0), Position::new(27, 30).adjacent(&map, &Direction::Down));
        assert_eq!(Position::new(0, 30), Position::new(27, 30).adjacent(&map, &Direction::Right));
    }

    #[test]
    fn can_calculate_manhattan_distance() {
        let map: Map = serde_json::from_str(DEFAULT_MAP).unwrap();

        // Non-wrapping
        assert_eq!(10, Position::new(10, 12).manhattan_distance_to(&Position::new(14, 18), &map));
        assert_eq!(10, Position::new(14, 18).manhattan_distance_to(&Position::new(10, 12), &map));
        assert_eq!(10, Position::new(18, 14).manhattan_distance_to(&Position::new(12, 10), &map));

        // Wrapping
        assert_eq!(1, Position::new(0, 0).manhattan_distance_to(&Position::new(0, 27), &map));
        assert_eq!(9, Position::new(25, 25).manhattan_distance_to(&Position::new(0, 0), &map));
        assert_eq!(9, Position::new(0, 0).manhattan_distance_to(&Position::new(25, 25), &map));
    }
}
