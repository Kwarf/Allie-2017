use common;

pub trait HasDimensions {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

pub trait HasPosition {
    fn position(&self) -> common::Position;
}
