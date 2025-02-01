use std::ops::Deref;

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

pub struct BakedNailTable<N: nails::Handle> {
    table: NailTable<N>,
    segments: Vec<BakedSegment<N::Scalar>>,
    distancer: NailDistancer,
}

impl<N: nails::Handle<Error: std::error::Error>> BakedNailTable<N> {
    pub fn new(table: NailTable<N>, min_nail_distance: usize) -> Result<Self, BakeError<N::Error>> {
        let nail_count = table.nails.len();
        let distancer =
            NailDistancer::new(nail_count, min_nail_distance).map_err(BakeError::Distancer)?;
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
                .map_err(BakeError::Nail)?,
            table,
            distancer,
        })
    }

    pub fn distancer(&self) -> &NailDistancer {
        &self.distancer
    }

    pub(crate) fn segments_mut(&mut self) -> &mut [BakedSegment<N::Scalar>] {
        &mut self.segments
    }
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
pub enum BakeError<N> {
    #[error(transparent)]
    Nail(N),
    #[error(transparent)]
    Distancer(nail_distancer::Error),
}
