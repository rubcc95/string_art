use crate::*;
use board::*;
use derive_more::*;
use geometry::Rect;
use nails::*;
use std::{
    f32::consts::PI,
    fmt::{Debug, Display},
};

#[derive(Debug, Clone)]
pub struct Ellipse<N: nails::Builder> {
    segments: Vec<Segment>,
    pub handle: N::Handle,
    min_nail_distance: usize,
    nail_count: usize,
    pub nails: Vec<N::Nail>,
}

impl<N: nails::Builder> IntoBoard for Ellipse<N> {
    type Board = EllipseBoard<N>;

    fn into_board(self, cpus: usize) -> Self::Board {
        let total_combs = self.nail_count - 2 * self.min_nail_distance - 1;
        EllipseBoard {
            ellipse: self,
            total_combs,
            per_batch: (total_combs + cpus - 1) / cpus,
        }
    }
}

impl<N: nails::Builder> Ellipse<N> {
    pub fn new(
        rect: Rect<f32>,
        nail_builder: N,
        nail_count: usize,
        min_nail_distance: usize,
    ) -> Result<Self, Error<N::Error>> {
        let nails: Vec<_> = (0..nail_count)
            .map(|i| {
                let theta = 2.0 * PI * i as f32 / nail_count as f32;
                nail_builder.create_nail(
                    geometry::Point {
                        x: 0.5 * rect.width * (1.0 + theta.cos()),
                        y: 0.5 * rect.height * (1.0 + theta.sin()),
                    },
                    theta,
                )
            })
            .collect();

        let nails_view = nails.as_slice();
        let builder = &nail_builder;

        let this = Self {
            segments: Self::get_anchors(nail_count, min_nail_distance)
                .map(|(big, small)| {
                    Ok(builder
                        .create_segment(                            
                            (unsafe { nails_view.get_unchecked(small.idx) }, small.link),
                            (unsafe { nails_view.get_unchecked(big.idx) }, big.link),
                        )?
                        .into())
                })
                .collect::<Result<Vec<_>, N::Error>>()
                .map_err(Error::Nail)?,
            handle: nail_builder.anchor_builder(),
            min_nail_distance,
            nail_count,
            nails,
        };
        Ok(this)
    }

    fn get_anchors(
        nail_count: usize,
        min_nail_distance: usize,
    ) -> impl Iterator<Item = (nails::Anchor<N::Link>, nails::Anchor<N::Link>)> {
        (0..nail_count)
            .into_iter()
            .map(move |big| {
                let dt_to_end = nail_count - big - 1; //1
                let range = min_nail_distance.saturating_sub(dt_to_end) //0..499
                    ..big.saturating_sub(min_nail_distance);
                range.into_iter().flat_map(move |small| {
                    N::LINKS.into_iter().flat_map(move |b_link| {
                        N::LINKS.into_iter().map(move |s_link| {
                            (
                                Anchor {
                                    idx: big,
                                    link: b_link.clone(),
                                },
                                Anchor {
                                    idx: small,
                                    link: s_link,
                                },
                            )
                        })
                    })
                })
            })
            .flatten()
    }

    fn index_of(&self, a: nails::Anchor<N::Link>, b: nails::Anchor<N::Link>) -> Option<usize> {
        let (mut big, mut small) = match a.idx.cmp(&b.idx) {
            std::cmp::Ordering::Less => (self.handle.reversed(b), self.handle.reversed(a)),
            std::cmp::Ordering::Equal => return None,
            std::cmp::Ordering::Greater => (a, b),
        };

        let cap = self.nail_count - (self.min_nail_distance + 1); //479
        let first = match big.idx.checked_sub(cap) {
            // 1
            Some(diff) => {
                big.idx = cap;
                small.idx = small.idx.checked_sub(diff)?;
                diff * (cap - self.min_nail_distance)
            }
            None => 0,
        };

        let diff = big.idx.checked_sub(self.min_nail_distance)?; // 1
        let index = first + diff * (diff - 1) / 2 + small.idx;
        Some(
            <N::Links as nails::Links>::SQ_LEN * index
                + <N::Links as nails::Links>::LEN * self.handle.index_of(&big.link)
                + self.handle.index_of(&small.link),
        )
    }
}

#[derive(Deref, DerefMut)]
pub struct EllipseBoard<B: nails::Builder> {
    #[deref]
    #[deref_mut]
    ellipse: Ellipse<B>,
    total_combs: usize,
    per_batch: usize,
}

impl<B: nails::Builder> Display for EllipseBoard<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ellipse, nail count: {}", self.ellipse.nail_count)
    }
}

impl<B: nails::Builder> Board for EllipseBoard<B> {
    type Anchor = nails::Anchor<B::Link>;

    fn get_segment(&self, idx: usize) -> Option<&Segment> {
        self.segments.get(idx)
    }

    fn get_segment_mut(&mut self, idx: usize) -> Option<&mut Segment> {
        self.ellipse.segments.get_mut(idx)
    }

    fn batch<'a>(
        &'a self,
        from: &'a Self::Anchor,
        index: usize,
    ) -> impl Iterator<Item: board::SegmentRef<Anchor = Self::Anchor>> + 'a {
        let start = index * self.per_batch;
        let range = start..self.total_combs.min(start + self.per_batch);
        range.flat_map(move |index| {
            let mut to_idx = 1 + from.idx + self.min_nail_distance + index;
            if let Some(new_idx) = to_idx.checked_sub(self.nail_count) {
                to_idx = new_idx;
            }

            B::LINKS.into_iter().map(move |to_link| {
                let to = nails::Anchor {
                    idx: to_idx,
                    link: to_link.clone(),
                };
                let index = match self.index_of(from.clone(), to.clone()) {
                    Some(data) => data,
                    None => panic!(
                        "Failed {}, {:?}, {}, {:?}",
                        from.idx,
                        self.handle.index_of(&from.link),
                        to.idx,
                        self.handle.index_of(&to.link)
                    ),
                };                
                SegmentRef::<B> {
                    segment: unsafe { self.segments.get_unchecked(index) },
                    next_anchor: self.handle.next_anchor(nails::Anchor {
                        idx: to_idx,
                        link: to_link,
                    }),
                    index,
                }
            })
        })
    }
}

pub struct SegmentRef<'a, N: nails::Builder> {
    segment: &'a Segment,
    next_anchor: nails::Anchor<N::Link>,
    index: usize,
}

impl<'a, N: nails::Builder> board::SegmentRef for SegmentRef<'a, N> {
    type Anchor = nails::Anchor<N::Link>;

    fn segment(&self) -> &Segment {
        self.segment
    }

    fn index(&self) -> usize {
        self.index
    }

    fn anchor(&self) -> Self::Anchor {
        self.next_anchor.clone()
    }
}

#[derive(Debug, thiserror::Error)]

pub enum Error<N> {
    #[error(transparent)]
    Nail(N),
    #[error("The minimum distance between nails must be smaller than {0}.")]
    Distancer(usize),
}

#[cfg(test)]
mod test {
    use crate::board::SegmentRef;

    use super::*;

    pub fn test_ellipse<N: nails::Builder>(
        nails: N,
        nail_count: usize,
        nail_distance: usize,
        batches: usize,
    ) {
        let ellipse =
            Ellipse::new(Rect::new(1000.0, 1000.0), nails, nail_count, nail_distance).unwrap();
        let board = ellipse.into_board(batches);
        let mut i = 0;
        for (a, b) in Ellipse::<N>::get_anchors(nail_count, nail_distance) {
            match board.ellipse.index_of(a.clone(), b.clone()) {
                Some(index) => {
                    assert_eq!(
                        i,
                        index,
                        "Expected index {}, but got {} for anchors: {:?} and {:?}",
                        i,
                        index,
                        anchor::Debugger {
                            handle: &board.handle,
                            anchor: &a
                        },
                        anchor::Debugger {
                            handle: &board.handle,
                            anchor: &b
                        }
                    );
                    i += 1;
                }
                None => panic!(
                    "Failed to get index for anchors: {:?} and {:?}",
                    anchor::Debugger {
                        handle: &board.handle,
                        anchor: &a
                    },
                    anchor::Debugger {
                        handle: &board.handle,
                        anchor: &b
                    }
                ),
            }
        }
        let mut counter = vec![0; board.ellipse.segments.len()];

        for anchor_idx in 0..nail_count {
            for link in N::LINKS.into_iter() {
                for batch_idx in 0..batches {
                    for item in EllipseBoard::batch(
                        &board,
                        &Anchor {
                            idx: anchor_idx,
                            link: link.clone(),
                        },
                        batch_idx,
                    ) {
                        let i = SegmentRef::index(&item);
                        let anchor = SegmentRef::anchor(&item);
                        assert!(
                            board.segments.len() > i,
                            "Overflow range from: ({}), to: ({}) -> {}. (Max value expected: {})",
                            anchor_idx,
                            anchor.idx,
                            i,
                            board.segments.len()
                        );
                        counter[i] += 1;
                    }
                }
            }
        }
        assert!(counter.iter().all(|&c| c == 2));
    }

    #[test]
    pub fn test() {
        test_ellipse(UniformCircular(0.5), 10, 1, 4);
        test_ellipse(Point, 123, 1, 10);
    }
}
