use std::fmt;

#[derive(PartialEq)]
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
}

#[derive(Clone, Eq, Hash, PartialEq)]
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

    pub fn manhattan_distance_to(&self, other: &Position) -> u32 {
        // So much typecasting that I don't even
        ((self.x as i32 - other.x as i32).abs() + (self.y as i32 - other.y as i32).abs()) as u32
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
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_calculate_manhattan_distance() {
        assert_eq!(50, Position::new(25, 25).manhattan_distance_to(&Position::new(0, 0)));

        assert_eq!(10, Position::new(10, 12).manhattan_distance_to(&Position::new(14, 18)));
        assert_eq!(10, Position::new(14, 18).manhattan_distance_to(&Position::new(10, 12)));
        assert_eq!(10, Position::new(18, 14).manhattan_distance_to(&Position::new(12, 10)));
    }
}
