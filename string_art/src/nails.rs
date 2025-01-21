pub mod circular;

use crate::Float;
pub use circular::Circular;
use svg::Node;

use crate::geometry::{Point, Segment};

pub trait Builder {
    type Scalar: Float;
    type Nail: Copy;
    type Handle: Handle<Scalar = Self::Scalar, Nail = Self::Nail>;

    fn build_nail(&self, position: Point<Self::Scalar>, rotation: Self::Scalar) -> Self::Nail;

    fn build_handle(self) -> Self::Handle;
}

pub unsafe trait Links: IntoIterator<Item = Self::Link> {
    type Link: Copy + Into<usize> + Send + Sync;

    const LEN: usize;
    const SQ_LEN: usize = Self::LEN * Self::LEN;
}

pub trait Handle: Copy + Send + Sync {
    type Scalar: Float;
    type Nail: Copy + Send + Sync;
    type Links: Links<Link = Self::Link>;
    type Link: Copy + Into<usize> + Send + Sync;
    type Error: std::error::Error;

    const LINKS: Self::Links;

    fn get_segment(
        self,
        start: (&Self::Nail, <Self::Links as IntoIterator>::Item),
        end: (&Self::Nail, <Self::Links as IntoIterator>::Item),
    ) -> Result<Segment<Self::Scalar>, Self::Error>;

    fn get_next_link(
        self,
        prev_link: <Self::Links as IntoIterator>::Item,
    ) -> <Self::Links as IntoIterator>::Item;

    fn draw_svg(self, nail: Self::Nail) -> impl Into<Box<dyn Node>>;
}
