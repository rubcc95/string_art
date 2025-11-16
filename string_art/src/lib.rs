pub mod computation; 
pub mod board;
pub mod ellipse;
pub mod nails;
pub mod decay;
pub mod monocolor;
pub mod multicolor {}
mod color_map;

pub use color_map::ColorMap;
pub use ellipse::Ellipse;
pub use board::Board;
pub use computation::Computation;
pub use monocolor::Monocolor;

pub use string_art_math as math;
pub use string_art_sync as sync;
pub use string_art_geometry as geometry;
pub use string_art_color as color;

pub use color::Color;