use super::{BakedSegment, NailTable};
use crate::{
    geometry::{Point, Rect},
    nails,
    verboser::{self, Message},
    Float, 
};
use num_traits::{AsPrimitive, ConstOne as _, Float as _};

pub struct Ellipse<N: nails::Handle> {
    nails: Vec<N::Nail>,
    handle: N,
    segments: Vec<BakedSegment<N::Scalar>>,
    distancer: NailDistancer,
}

unsafe impl<N: nails::Handle> NailTable for Ellipse<N> {
    type Handle = N;
    type Id = usize;

    fn nails(&self) -> &[<Self::Handle as nails::Handle>::Nail] {
        &self.nails
    }

    unsafe fn comb_count(&self, _: usize) -> usize {
        self.distancer.distance()
    }

    unsafe fn segment_idx(
        &mut self,
        nail_idx: usize,
        link: <Self::Handle as nails::Handle>::Link,
        offset: usize,
        other_link: <Self::Handle as nails::Handle>::Link,
    ) -> (
        usize,
        &mut BakedSegment<<Self::Handle as nails::Handle>::Scalar>,
    ) {
        let total = nail_idx + offset;
        let other = if total < self.nails.len() {
            total
        } else {
            total.unchecked_sub(self.nails.len())
        };
        (
            other,
            self.segments.get_unchecked_mut(
                self.distancer
                    .index_of_unchecked::<<Self::Handle as nails::Handle>::Links>(
                        nail_idx, link, other, other_link,
                    ),
            ),
        )
    }

    fn handle(&self) -> &Self::Handle {
        &self.handle
    }

    fn is_valid(&self, id: Self::Id) -> bool {
        id < self.nails.len()
    }
}

// impl<N: nails::Handle> From<BakedNailTable<N>> for NailTable2<N> {
//     fn from(value: BakedNailTable<N>) -> Self {
//         value.table
//     }
// }

impl<N: nails::Handle<Error: std::error::Error>> Ellipse<N> {
    pub fn new<B: nails::Builder<Scalar: Float, Handle = N, Nail = N::Nail>>(
        rect: Rect,
        nail_builder: B,
        nail_count: usize,
        verboser: &mut impl verboser::Verboser,
        min_nail_distance: usize,
    ) -> Result<Self, Error<N::Error>>
    where
        usize: AsPrimitive<B::Scalar>,
    {
        let nails: Vec<N::Nail> = (0..nail_count)
            .into_iter()
            .map(|i| {
                verboser.verbose(Message::CreatingNail(i));
                let theta: B::Scalar =
                    B::Scalar::TWO * B::Scalar::PI * (i.as_()) / (nail_count.as_());
                nail_builder.build_nail(
                    Point {
                        x: rect.width.as_() * (B::Scalar::ONE + theta.cos()),
                        y: rect.height.as_() * (B::Scalar::ONE + theta.sin()),
                    } * B::Scalar::HALF,
                    theta,
                )
            })
            .collect();
        verboser.verbose(Message::CreatingNail(nail_count));
        let nail_count = nails.len();
        let distancer = NailDistancer::new(nail_count, min_nail_distance)?;
        let handle = nail_builder.build_handle();
        let nails_ref = nails.as_slice();

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
                                                segment: handle.get_segment(
                                                    (
                                                        unsafe {
                                                            nails_ref.get_unchecked(big_idx)
                                                        },
                                                        big_link,
                                                    ),
                                                    (
                                                        unsafe {
                                                            nails_ref.get_unchecked(small_idx)
                                                        },
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
            nails,
            handle,
            distancer,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<N> {
    #[error(transparent)]
    Nail(N),
    #[error("The minimum distance between nails must be smaller than {0}.")]
    Distancer(usize),
}

#[derive(Clone, Copy)]
struct NailDistancer {
    min: usize,
    max: usize,
}

impl NailDistancer {
    fn new<N>(count: usize, distance: usize) -> Result<Self, Error<N>> {
        if count < 2 * distance {
            Err(Error::Distancer((1 + count) / 2))
        } else {
            Ok(Self {
                min: distance,
                //SAFETY: checked bounds in if
                max: unsafe { count.unchecked_sub(distance) },
            })
        }
    }

    // pub fn min(&self) -> usize{
    //     self.min
    // }

    fn distance(&self) -> usize {
        unsafe { self.max.unchecked_sub(self.min) }
    }

    //NOTE: This does not check if a_idx and b_idx are inside bounds.
    // a_idx and b_idx must be also inside bounds in order to be valid.
    fn is_valid(&self, a_idx: usize, b_idx: usize) -> bool {
        let diff = a_idx.abs_diff(b_idx);
        diff > self.min && diff < self.max
    }

    // pub fn index_of<L: nails::Links>(
    //     &self,
    //     a_idx: usize,
    //     a_link: L::Item,
    //     b_idx: usize,
    //     b_link: L::Item,
    // ) -> Option<usize> {
    //     if self.is_valid(a_idx, b_idx) {
    //         Some(unsafe { self.index_of_unchecked::<L>(a_idx, a_link, b_idx, b_link) })
    //     } else {
    //         None
    //     }
    // }

    //SAFETY: caller must ensure that the indices are valid via NailDistancer::is_valid
    unsafe fn index_of_unchecked<L: nails::Links>(
        &self,
        a_idx: usize,
        a_link: L::Item,
        b_idx: usize,
        b_link: L::Item,
    ) -> usize {
        let (mut big_idx, big_link, mut small_idx, small_link) = if a_idx > b_idx {
            (a_idx, a_link, b_idx, b_link)
        } else {
            (b_idx, b_link, a_idx, a_link)
        };
        let cap = self.max.unchecked_sub(1);
        let first = if big_idx > cap {
            let diff = big_idx.unchecked_sub(cap);
            big_idx = cap;
            small_idx = small_idx.unchecked_sub(diff);
            diff.unchecked_mul(cap.unchecked_sub(self.min))
                .unchecked_mul(L::SQ_LEN)
        } else {
            0
        };

        let diff = big_idx.unchecked_sub(self.min);

        first
            .unchecked_add((diff * diff.unchecked_sub(1) / 2).unchecked_mul(L::SQ_LEN))
            .unchecked_add(L::LEN.unchecked_mul(diff).unchecked_mul(big_link.into()))
            .unchecked_add(L::LEN.unchecked_mul(small_idx))
            .unchecked_add(small_link.into())
    }
}
