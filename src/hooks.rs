pub mod circular;

pub use circular::Circular;

use crate::geometry::{Point, Segment};

pub trait Builder<T> {
    type Hook;
    type Handle;

    fn build_hook(&self, position: Point<T>, rotation: T) -> Self::Hook;

    fn build_handle(self) -> Self::Handle;
}

pub unsafe trait Links: IntoIterator<Item: Copy + Into<usize>> {
    const LEN: usize;
    const SQ_LEN: usize = Self::LEN * Self::LEN;
}

pub trait Handle<T> {
    type Hook: Copy;
    type Links: Links;

    const LINKS: Self::Links;

    fn get_segment(
        &self,
        start: (&Self::Hook, <Self::Links as IntoIterator>::Item),
        end: (&Self::Hook, <Self::Links as IntoIterator>::Item),
    ) -> Segment<T>;

    fn get_next_link(
        &self,
        prev_link: <Self::Links as IntoIterator>::Item,
    ) -> <Self::Links as IntoIterator>::Item;
}
