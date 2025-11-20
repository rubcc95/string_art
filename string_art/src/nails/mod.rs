pub mod anchor;
pub mod circular;
pub mod point;

pub use anchor::Anchor;
pub use circular::UniformCircular;
pub use point::Point;

use crate::geometry;
//use crate::sync::*;

pub trait Builder {
    type Nail;
    type Links: Links<Link = Self::Link>;
    type Handle: Handle<Link = Self::Link, Nail = Self::Nail>;
    type Link: Copy + Default;
    type Error: std::error::Error;

    const LINKS: Self::Links;

    fn create_nail(&self, position: geometry::Point<f32>, rotation: f32) -> Self::Nail;

    fn create_segment(
        &self,
        start: (&Self::Nail, Self::Link),
        end: (&Self::Nail, Self::Link),
    ) -> Result<geometry::Segment<f32>, Self::Error>;

    fn anchor_builder(self) -> Self::Handle;
}

pub unsafe trait Links: IntoIterator<Item = Self::Link> {
    type Link;

    const LEN: usize;
    const SQ_LEN: usize = Self::LEN * Self::LEN;
}

pub trait Handle {
    type Nail;
    type Link: Default;

    fn next_anchor(&self, anchor: Anchor<Self::Link>) -> Anchor<Self::Link>;

    fn index_of(&self, link: &Self::Link) -> usize;

    fn reversed(&self, anchor: Anchor<Self::Link>) -> Anchor<Self::Link>;
}
