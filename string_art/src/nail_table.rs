use std::ops::{Deref, Range};

use num_traits::{AsPrimitive, ConstOne as _, Float as _};

use crate::{
    geometry::{Point, Segment},
    grid::Grid,
    nail_distancer::{self, NailDistancer},
    nails,
    verboser::{self, Message},
    Float,
};

#[derive(Clone)]
pub struct NailTable<N: nails::Handle> {
    nails: Vec<N::Nail>,
    handle: N,
}

impl<N: nails::Handle> NailTable<N> {
    pub fn square<B: nails::Builder<Scalar: Float, Handle = N, Nail = N::Nail>>(
        grid: Grid,
        nail_builder: B,
        mut nail_count: usize,
        _: &mut impl verboser::Verboser,
    ) -> Result<Self, SquareTableError>
    where
        usize: AsPrimitive<B::Scalar>,
    {
        if nail_count == 0 {
            return Err(SquareTableError::MinNailCount);
        }
        if nail_count % 4 != 0 {
            return Err(SquareTableError::NotMultipleOf4);
        }
        let offset = nail_builder.offset();

        let mut nails = Vec::with_capacity(nail_count);
        nail_count = unsafe { nail_count.unchecked_sub(4) } / 2;

        let start = Point {
            x: -offset - B::Scalar::EPSILON,
            y: -offset - B::Scalar::EPSILON,
        };
        let end = Point::from(grid).as_()
            + Point {
                x: offset,
                y: offset,
            };
        let size = end - start;

        let x_count = num_traits::ToPrimitive::to_usize(
            &((((nail_count + 1).as_() * size.x - size.y) / (size.x + size.y)).round()),
        )
        .unwrap_or(0)
            + 2;
        let y_count = nail_count + 4 - x_count;

        nails.push(nail_builder.build_nail(
            Point {
                x: start.x,
                y: start.y,
            },
            B::Scalar::FRAC_5PI_4,
        ));
        nails.extend((1..x_count).map(|idx| {
            nail_builder.build_nail(
                Point {
                    x: start.x + size.x * idx.as_() / x_count.as_(),
                    y: start.y,
                },
                B::Scalar::FRAC_3PI_2,
            )
        }));
        nails.push(nail_builder.build_nail(
            Point {
                x: end.x,
                y: start.y,
            },
            B::Scalar::FRAC_7PI_4,
        ));
        nails.extend((1..y_count).map(|idx| {
            nail_builder.build_nail(
                Point {
                    x: end.x,
                    y: start.y + size.y * idx.as_() / y_count.as_(),
                },
                <B::Scalar as num_traits::ConstZero>::ZERO,
            )
        }));
        nails.push(nail_builder.build_nail(Point { x: end.x, y: end.y }, B::Scalar::FRAC_PI_4));
        nails.extend((1..x_count).rev().map(|idx| {
            nail_builder.build_nail(
                Point {
                    x: start.x + size.x * idx.as_() / x_count.as_(),
                    y: end.y,
                },
                B::Scalar::FRAC_PI_2,
            )
        }));
        nails.push(nail_builder.build_nail(
            Point {
                x: start.x,
                y: end.y,
            },
            B::Scalar::FRAC_3PI_4,
        ));
        nails.extend((1..y_count).rev().map(|idx| {
            nail_builder.build_nail(
                Point {
                    x: start.x,
                    y: start.y + size.y * idx.as_() / y_count.as_(),
                },
                B::Scalar::PI,
            )
        }));
        Ok(Self {
            nails,
            handle: nail_builder.build_handle(),
        })
    }

    pub fn ellipse<B: nails::Builder<Scalar: Float, Handle = N, Nail = N::Nail>>(
        grid: Grid,
        nail_builder: B,
        nail_count: usize,
        verboser: &mut impl verboser::Verboser,
    ) -> Self
    where
        usize: AsPrimitive<B::Scalar>,
    {
        let nails = (0..nail_count)
            .into_iter()
            .map(|i| {
                verboser.verbose(Message::CreatingNail(i));
                let theta: B::Scalar =
                    B::Scalar::TWO * B::Scalar::PI * (i.as_()) / (nail_count.as_());
                nail_builder.build_nail(
                    Point {
                        x: grid.width.as_() * (B::Scalar::ONE + theta.cos()),
                        y: grid.height.as_() * (B::Scalar::ONE + theta.sin()),
                    } * B::Scalar::HALF,
                    theta,
                )
            })
            .collect();
        verboser.verbose(Message::CreatingNail(nail_count));
        Self {
            nails,
            handle: nail_builder.build_handle(),
        }
    }

    pub fn handle(&self) -> N {
        self.handle
    }

    pub fn nails(&self) -> &[N::Nail] {
        &self.nails
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SquareTableError {
    #[error("Nail count must be greater or equal to 4")]
    MinNailCount,
    #[error("Nail count must be multiple of 4")]
    NotMultipleOf4,
}

/// The `Baked` trait is an unsafe trait that provides methods for working with a baked nail table.
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
pub unsafe trait Baked: Send + Sync {
    type Handle: nails::Handle;

    /// Returns a slice of nails.
    fn nails(&self) -> &[<Self::Handle as nails::Handle>::Nail];

    /// Returns a range of combination counts for a given nail index.
    ///
    /// # Safety
    ///
    /// `nail_idx` must be in the range 0..[`Self::nails`].[`len()`].
    ///
    /// [`len()`]: https://doc.rust-lang.org/std/primitive.slice.html#method.len
    unsafe fn comb_count(&self, nail_idx: usize) -> Range<usize>;

    /// Returns the segment index for a given nail index, link, offset, and other link.
    ///
    /// # Safety
    ///
    /// - `nail_idx` must be in the range 0..[`Self::nails`].[`len()`].
    /// - `offset` must be in the range [`Self::comb_count`].
    ///
    /// [`len()`]: https://doc.rust-lang.org/std/primitive.slice.html#method.len
    unsafe fn segment_idx(
        &self,
        nail_idx: usize,
        link: <Self::Handle as nails::Handle>::Link,
        offset: usize,
        other_link: <Self::Handle as nails::Handle>::Link,
    ) -> (usize, usize);

    /// Returns a mutable slice of baked segments.
    fn segments(&mut self) -> &mut [BakedSegment<<Self::Handle as nails::Handle>::Scalar>];

    /// Returns the nail handle associated to this baked instance.
    fn handle(&self) -> &Self::Handle;
}

unsafe impl<N: nails::Handle> Baked for BakedNailTable<N> {
    type Handle = N;

    fn nails(&self) -> &[<Self::Handle as nails::Handle>::Nail] {
        &self.nails
    }

    unsafe fn comb_count(&self, _: usize) -> Range<usize> {
        0..self.distancer.distance()
    }

    unsafe fn segment_idx(
        &self,
        nail_idx: usize,
        link: <Self::Handle as nails::Handle>::Link,
        offset: usize,
        other_link: <Self::Handle as nails::Handle>::Link,
    ) -> (usize, usize) {
        let total = nail_idx + offset;
        let other = if total < self.nails.len() {
            total
        } else {
            total.unchecked_sub(self.nails.len())
        };
        (
            other,
            self.distancer
                .index_of_unchecked::<<Self::Handle as nails::Handle>::Links>(
                    nail_idx, link, other, other_link,
                ),
        )
    }

    fn segments(&mut self) -> &mut [BakedSegment<<Self::Handle as nails::Handle>::Scalar>] {
        &mut self.segments
    }

    fn handle(&self) -> &Self::Handle {
        &self.table.handle
    }
}

pub struct BakedNailTable<N: nails::Handle> {
    table: NailTable<N>,
    segments: Vec<BakedSegment<N::Scalar>>,
    distancer: NailDistancer,
}

impl<N: nails::Handle> From<BakedNailTable<N>> for NailTable<N> {
    fn from(value: BakedNailTable<N>) -> Self {
        value.table
    }
}

impl<N: nails::Handle<Error: std::error::Error>> BakedNailTable<N> {
    pub fn new(table: NailTable<N>, min_nail_distance: usize) -> Result<Self, Error<N::Error>> {
        let nail_count = table.nails.len();
        let distancer =
            NailDistancer::new(nail_count, min_nail_distance).map_err(Error::Distancer)?;
        let nails = &table.nails;

        Ok(Self {
            segments: (0..nail_count)
                .into_iter()
                .map(move |big_idx| {
                    N::LINKS
                        .into_iter()
                        .map(move |big_link| {
                            (0..big_idx)
                                .filter_map(move |small_idx| {
                                    if distancer.is_valid(big_idx, small_idx) {
                                        Some(N::LINKS.into_iter().map(move |small_link| {
                                            Ok(BakedSegment {
                                                segment: table.handle.get_segment(
                                                    (
                                                        unsafe { nails.get_unchecked(big_idx) },
                                                        big_link,
                                                    ),
                                                    (
                                                        unsafe { nails.get_unchecked(small_idx) },
                                                        small_link,
                                                    ),
                                                )?,
                                                used: false,
                                            })
                                        }))
                                    } else {
                                        None
                                    }
                                })
                                .flatten()
                        })
                        .flatten()
                })
                .flatten()
                .collect::<Result<_, _>>()
                .map_err(Error::Nail)?,
            table,
            distancer,
        })
    }

    // pub fn distancer(&self) -> &NailDistancer {
    //     &self.distancer
    // }

    // pub(crate) fn segments_mut(&mut self) -> &mut [BakedSegment<N::Scalar>] {
    //     &mut self.segments
    // }
}

impl<N: nails::Handle> Deref for BakedNailTable<N> {
    type Target = NailTable<N>;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
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

impl<S> From<BakedSegment<S>> for Segment<S>{
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

#[derive(Debug, thiserror::Error)]
pub enum Error<N> {
    #[error(transparent)]
    Nail(N),
    #[error(transparent)]
    Distancer(nail_distancer::Error),
}
