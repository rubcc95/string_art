pub mod board;
pub mod computation;
pub mod decay;
pub mod ellipse;
pub mod monocolor;
pub mod nails;
pub mod multicolor {}
mod draw_backend;
pub mod pipeline;

pub use board::Board;
pub use color::*;
pub use computation::{Computation, Step};
pub use draw_backend::DrawBackend;
pub use ellipse::Ellipse;
pub use monocolor::Monocolor;
pub use pipeline::*;
pub use string_art_color as color;
pub use string_art_geometry as geometry;
pub use string_art_math as math;
