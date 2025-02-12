use std::ops::Deref;

pub mod rectangle;
pub mod ellipse;

pub use rectangle::Rectangle;
pub use ellipse::Ellipse;

use crate::{geometry::Segment, nails};

/// The `NailTable` trait is an unsafe trait that provides methods for working with a baked nail table.
/// Implementors of this trait must ensure certain invariants are upheld, as described in the safety
/// comments for each method.
///
/// # Safety
///
/// - [`Self::nails`].[`len()`] must be constant.
/// - [`Self::segments()`].[`len()`] and [`Self::segments_mut()`].[`len()`] must be constant and equal.
/// - `[Self::comb_count()]` must be constant for valid arguments.
/// - The return value of [`Self::segment_idx`] must be within the ranges
///   (0..[`Self::nails(&self)`].[`len()`], 0..[`Self::segments()`].[`len()`]) for valid arguments.
///
/// [`len()`]: https://doc.rust-lang.org/std/primitive.slice.html#method.len
pub unsafe trait NailTable: Send + Sync {
    type Handle: nails::Handle;
    type Id: Send + Sync + ToString + Copy + Default;

    /// Returns a slice of nails.
    fn nails(&self) -> &[<Self::Handle as nails::Handle>::Nail];

    /// Returns a range of combination counts for a given nail index.
    ///
    /// # Safety
    ///
    /// `nail_idx` must be in the range 0..[`Self::nails`].[`len()`].
    ///
    /// [`len()`]: https://doc.rust-lang.org/std/primitive.slice.html#method.len
    unsafe fn comb_count(&self, nail_idx: Self::Id) -> usize;

    /// Returns the segment index for a given nail index, link, offset, and other link.
    ///
    /// # Safety
    ///
    /// - `nail_idx` must be in the range 0..[`Self::nails`].[`len()`].
    /// - `offset` must be in the range 0..[`Self::comb_count`].
    ///
    /// [`len()`]: https://doc.rust-lang.org/std/primitive.slice.html#method.len
    unsafe fn segment_idx(
        &mut self,
        nail_idx: Self::Id,
        link: <Self::Handle as nails::Handle>::Link,
        offset: usize,
        other_link: <Self::Handle as nails::Handle>::Link,
    ) -> (
        Self::Id,
        &mut BakedSegment<<Self::Handle as nails::Handle>::Scalar>,
    );

    fn is_valid(&self, id: Self::Id) -> bool;

    // /// Returns a mutable slice of baked segments.
    // fn segments(&mut self) -> &mut [BakedSegment<<Self::Handle as nails::Handle>::Scalar>];

    /// Returns the nail handle associated to this baked instance.
    fn handle(&self) -> &Self::Handle;
}

#[derive(Clone, Copy)]
pub struct BakedSegment<S> {
    segment: Segment<S>,
    used: bool,
}

impl<S> From<Segment<S>> for BakedSegment<S> {
    fn from(segment: Segment<S>) -> Self {
        Self {
            segment,
            used: false,
        }
    }
}

impl<S> From<BakedSegment<S>> for Segment<S> {
    fn from(value: BakedSegment<S>) -> Self {
        value.segment
    }
}

impl<S> BakedSegment<S> {
    pub fn segment(&self) -> &Segment<S> {
        &self.segment
    }

    pub fn mark_used(&mut self) {
        self.used = true;
    }

    pub fn is_used(&self) -> bool {
        self.used
    }
}

impl<S> Deref for BakedSegment<S> {
    type Target = Segment<S>;

    fn deref(&self) -> &Self::Target {
        &self.segment
    }
}

