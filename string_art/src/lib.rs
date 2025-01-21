//mod algorithm;
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
mod algorithm;
//mod algorithm_safe_copy;
use grid::Grid;
mod nail_table;
//pub use algorithm::Error;
pub mod color;
mod color_map;
pub mod line_selector;
mod nail_distancer;

pub mod auto_line_config;
pub mod darkness;
pub mod line_config;

pub use algorithm::*;
pub use auto_line_config::AutoLineConfig;
pub use color::*;
pub use color_map::ColorConfig;
pub use image::*;
pub use line_config::LineConfig;
//pub use line_selector::{Builder as LineSelectorBuilder, Error as LineSelectorError, LineSelector};
pub use nail_table::NailTable;
pub mod verboser;
