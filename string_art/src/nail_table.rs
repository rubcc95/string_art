use std::ops::Deref;

use num_traits::{AsPrimitive, ConstOne, Float as _};

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

pub struct BakedNailTable<N: nails::Handle> {
    table: NailTable<N>,
    segments: Vec<BakedSegment<N::Scalar>>,
    distancer: NailDistancer,
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
                                            //((big_idx, big_link), (small_idx, small_link))
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
pub enum Error<N> {
    #[error(transparent)]
    Nail(N),
    #[error(transparent)]
    Distancer(nail_distancer::Error),
}
