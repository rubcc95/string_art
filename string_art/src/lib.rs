pub mod geometry {
    pub mod circle;
    pub mod point;
    pub mod segment;

    pub use circle::Circle;
    pub use point::Point;
    pub use segment::Segment;
}

mod algorithm;
pub mod color;
pub mod darkness;
mod float;
pub mod grid;
pub mod image;
mod nail_distancer;
mod nail_table;
pub mod nails;
pub mod slice;
pub mod verboser;

pub use algorithm::*;
pub use darkness::Darkness;
pub use float::Float;
pub use grid::Grid;
pub use image::Image;
pub use nail_table::*;
