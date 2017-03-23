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
