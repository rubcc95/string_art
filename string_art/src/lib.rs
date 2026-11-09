#![forbid(unsafe_code)]

pub mod board;
pub mod computation;
pub mod decay;
pub mod ellipse;
pub mod monocolor;
pub mod nails;
mod color_map;
mod num;

pub use board::Board;
pub use color_map::ColorMap;
pub use computation::Computation;
pub use ellipse::Ellipse;
pub use monocolor::Monocolor;
pub use num::{Frac, Scalar};

pub use string_art_color as color;
pub use string_art_geometry as geometry;
pub use string_art_sync as sync;

pub use color::Color;
