mod anchor;
pub mod circular;
mod point;

pub use anchor::Anchor;
pub use circular::Circular;
pub use point::Point;

use crate::geometry;
use crate::sync::*;

pub trait Builder {
    #[cfg(feature = "multithreaded")]
    type Nail: Sync;
    #[cfg(not(feature = "multithreaded"))]
    type Nail;
    type Links: Links<Link = Self::Link>;
    type Handle: Handle<Link = Self::Link, Nail = Self::Nail>;

    #[cfg(feature = "multithreaded")]
    type Link: Clone + Sync + Send + Default;
    #[cfg(not(feature = "multithreaded"))]
    type Link: Clone + Default;

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

/// Set of links that leave a nail.
///
/// The usual implementation is an array —`[Direction; 2]`, `[Link; 1]`—, which
/// takes `LEN` from its own length. If someone implements this by hand:
///
/// # Contract
///
/// * `LEN` must match the number of elements the iterator produces.
/// * `Handle::index_of` must number those links from `0` to `LEN - 1`, without
///   repeating and **in the same order in which the iterator traverses them**.
///
/// The second point is not cosmetic: `EllipseBoard::index_of` uses that number as
/// a coordinate inside a block of `LEN * LEN` segments. A mismatch
/// produces wrong indexes; access to the segment is checked, so
/// that is a panic and not an out-of-range read. `Ellipse::new` verifies it
/// in debug.
pub trait Links: IntoIterator<Item = Self::Link> {
    type Link;

    const LEN: usize;
    const SQ_LEN: usize = Self::LEN * Self::LEN;
}

/// The length is carried by the type, so `LEN` cannot disagree with the number of
/// elements the iterator produces. This is what replaces the former
/// `unsafe impl`.
impl<L, const N: usize> Links for [L; N] {
    type Link = L;

    const LEN: usize = N;
}

pub trait Handle: CondSync {
    type Nail: CondSync;
    type Link: Default;

    fn next_anchor(&self, anchor: Anchor<Self::Link>) -> Anchor<Self::Link>;

    fn index_of(&self, link: &Self::Link) -> usize;

    fn reversed(&self, anchor: Anchor<Self::Link>) -> Anchor<Self::Link>;
}
