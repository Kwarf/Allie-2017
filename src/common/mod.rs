use std::fmt;

use traits::HasDimensions;

#[derive(Clone, PartialEq)]
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

    // Returns neighbour position in provided direction, with limit wrapping
    // And yes, it's not very nice looking, but I think it works
    pub fn neighbour<T: HasDimensions>(&self, limits: &T, direction: &Direction) -> Position {
        let width = limits.width() - 1;
        let height = limits.height() - 1;
        match *direction {
            Direction::Up => Position {
                x: self.x,
                y: if self.y == 0 {
                    height
                }
                else {
                    self.y - 1
                },
            },
            Direction::Down => Position {
                x: self.x,
                y: if self.y == height {
                    0
                }
                else {
                    self.y + 1
                },
            },
            Direction::Left => Position {
                x: if self.x == 0 {
                    width
                }
                else {
                    self.x - 1
                },
                y: self.y
            },
            Direction::Right => Position {
                x: if self.x == width {
                    0
                }
                else {
                    self.x + 1
                },
                y: self.y,
            },
        }
    }

    pub fn neighbours<T: HasDimensions>(&self, limits: &T) -> [Position; 4] {
        [
            self.neighbour(limits, &Direction::Up),
            self.neighbour(limits, &Direction::Down),
            self.neighbour(limits, &Direction::Left),
            self.neighbour(limits, &Direction::Right),
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

    #[test]
    fn can_calculate_manhattan_distance() {
        assert_eq!(50, Position::new(25, 25).manhattan_distance_to(&Position::new(0, 0)));

        assert_eq!(10, Position::new(10, 12).manhattan_distance_to(&Position::new(14, 18)));
        assert_eq!(10, Position::new(14, 18).manhattan_distance_to(&Position::new(10, 12)));
        assert_eq!(10, Position::new(18, 14).manhattan_distance_to(&Position::new(12, 10)));
    }
}
