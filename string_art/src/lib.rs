#![forbid(unsafe_code)]

pub mod board;
pub mod computation;
pub mod ellipse;
pub mod nails;
pub mod walk;

mod decay;
mod monocolor;
mod num;

pub use string_art_color as color;
pub use string_art_geometry as geometry;

pub use board::Board;
pub use color::Color;
pub use computation::Computation;
pub use decay::Decay;
pub use ellipse::Ellipse;
pub use monocolor::Monocolor;
pub use num::{Frac, Scalar};
pub use string_art_executor::*;
#[cfg(feature = "clip")]
pub use walk::Clip;
pub use walk::PixelWalk;
#[cfg(feature = "cached")]
pub use walk::{CacheTooLarge, Cached};
