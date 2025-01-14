use num_traits::{AsPrimitive, ConstOne, Float as _};

use crate::{geometry::{Point, Segment}, grid::Grid, nail_distancer::{NailDistanceError, NailDistancer}, nails, Float, Image};

#[derive(Clone)]
pub struct Table<S, N: nails::Handle> {
    pub(crate) nails: Vec<N::Nail>,
    pub(crate) handle: N,
    pub image: Image<S>,
}

impl<S: Float, N: nails::Handle> Table<S, N>
where
    usize: AsPrimitive<S>,
{
    pub fn ellipse(
        image: impl Into<Image<S>>,
        nail_builder: impl nails::Builder<Scalar = S, Handle = N, Nail = N::Nail>,
        nail_count: usize,
    ) -> Self {
        let image = image.into();
        Self {
            nails: (0..nail_count)
                .into_iter()
                .map(|i| {
                    let theta: S = S::TWO * S::PI * (i.as_()) / (nail_count.as_());
                    nail_builder.build_nail(
                        Point {
                            x: image.width.as_() * (S::ONE + theta.cos()),
                            y: image.height.as_() * (S::ONE + theta.sin()),
                        } * S::HALF,
                        theta,
                    )
                })
                .collect(),
            handle: nail_builder.build_handle(),
            image,
        }
    }
}
#[derive(Clone)]
pub struct NailTable<N: nails::Handle> {
    pub(crate) nails: Vec<N::Nail>,
    pub(crate) handle: N,
}

impl<N: nails::Handle> NailTable<N> {
    pub fn ellipse<B: nails::Builder<Scalar: Float, Handle = N, Nail = N::Nail>>(
        grid: Grid,
        nail_builder: B,
        nail_count: usize,
    ) -> Self
    where
        usize: AsPrimitive<B::Scalar>,
    {
        Self {
            nails: (0..nail_count)
                .into_iter()
                .map(|i| {
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
                .collect(),
            handle: nail_builder.build_handle(),
        }
    }
}

pub struct BakedNailTable<N: nails::Handle>{
    table: NailTable<N>,   
    segments: Vec<BakedSegment<N::Scalar>>,
    distancer: NailDistancer,
}

impl<N: nails::Handle> BakedNailTable<N>{
    pub fn new(table: NailTable<N>, min_nail_distance: usize) -> Result<Self, NailDistanceError>{
        let nail_count = table.nails.len();
        let distancer = NailDistancer::new(nail_count, min_nail_distance)?;
        let nails = &table.nails;
        Ok(Self{
            segments: (0..nail_count).into_iter()
            .map(move |big_idx| {
                N::LINKS
                    .into_iter()
                    .map(move |big_link| {
                        (0..big_idx)
                            .filter_map(move |small_idx| {
                                if distancer.is_valid(big_idx, small_idx) {
                                    Some(N::LINKS.into_iter().map(move |small_link| {
                                        //((big_idx, big_link), (small_idx, small_link))
                                        BakedSegment {
                                            segment: table.handle.get_segment(
                                                (unsafe { nails.get_unchecked(big_idx) }, big_link),
                                                (unsafe { nails.get_unchecked(small_idx) }, small_link),
                                            ),
                                            used: false,
                                        }
                                    }))
                                } else {
                                    None
                                }
                            })
                            .flatten()
                    })
                    .flatten()
            })
            .flatten().collect(),
            table,
            distancer,
        })
    } 
}

#[derive(Clone, Copy)]
struct BakedSegment<S> {
    segment: Segment<S>,
    used: bool,
}

