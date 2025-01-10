mod float;
mod core;
mod ditherer;
mod colors;

pub mod grid;
pub mod nails;

pub use float::Float;

pub mod geometry {
    pub mod circle;
    pub mod point;
    pub mod segment;

    pub use circle::Circle;
    pub use point::Point;
    pub use segment::Segment;
}

use grid::Grid;
pub use core::*;
pub use colors::*;

//mod core_2;