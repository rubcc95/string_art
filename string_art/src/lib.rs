#![forbid(unsafe_code)]

pub mod board;
pub mod computation;
pub mod ellipse;
pub mod nails;

mod decay;
mod monocolor;
mod num;

pub use string_art_color as color;
pub use string_art_geometry as geometry;

use string_art_sync as sync;

pub use board::Board;
pub use color::Color;
pub use computation::Computation;
pub use decay::Decay;
pub use ellipse::Ellipse;
pub use monocolor::Monocolor;
pub use num::{Frac, Scalar};
pub use sync::Executor;
