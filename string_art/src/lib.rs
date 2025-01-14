mod algorithm;
mod ditherer;
mod float;
mod image;

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
mod algorithm_safe_copy;
pub use algorithm::*;
use grid::Grid;
pub use image::*;
mod nail_table;
pub use algorithm::Error;
pub mod color;
mod color_groups;
mod color_map;
mod nail_distancer;
pub use color::*;
