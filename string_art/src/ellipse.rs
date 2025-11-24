use crate::*;
use board::*;
use geometry::Rect;
use nails::*;
use std::{f32::consts::PI, ops::Range};

#[derive(Debug, Clone)]
pub struct Ellipse<N: nails::Builder> {
    pub nails: Vec<N::Nail>,
    pub nail_builder: N,
}

impl<N: nails::Builder> Ellipse<N> {
    pub fn new(rect: Rect<f32>, nail_builder: N, nail_count: usize) -> Self {
        Self {
            nails: (0..nail_count)
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
                .collect(),
            nail_builder,
        }
    }
}
pub struct Board<N: nails::Builder> {
    lines: Vec<Line>,
    pub handle: N::Handle,
    min_nail_distance: usize,
    nail_count: usize,
}

impl<N: nails::Builder> Board<N> {
    pub fn new(ellipse: &Ellipse<N>, min_nail_distance: usize) -> Result<Self, Error<N::Error>> {
        let nails_view = ellipse.nails.as_slice();
        let builder = &ellipse.nail_builder;
        let nail_count = ellipse.nails.len();
        let this = Self {
            lines: Self::get_anchors(nail_count, min_nail_distance)
                .map(|(big, small)| {
                    Ok(builder
                        .link_nails(
                            (unsafe { nails_view.get_unchecked(small.idx) }, small.link),
                            (unsafe { nails_view.get_unchecked(big.idx) }, big.link),
                        )?
                        .into())
                })
                .collect::<Result<Vec<_>, N::Error>>()
                .map_err(Error::Nail)?,
            handle: builder.to_handle(),
            min_nail_distance,
            nail_count,
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

impl<B: nails::Builder> crate::Board for Board<B> {
    type Batch = Range<usize>;

    type Anchor = nails::Anchor<B::Link>;

    type LineId = usize;

    fn get_batches(&self, cpus: usize) -> impl Iterator<Item = Self::Batch> {
        let total_combs = self.nail_count - 2 * self.min_nail_distance - 1;
        let per_batch = (total_combs + cpus - 1) / cpus;

        let mut start = 0;

        (0..cpus).map(move |_| {
            let end = total_combs.min(start + per_batch);
            let res = start..end;
            start = end;
            res
        })
    }

    fn get_indexes(
        &self,
        batch: &Self::Batch,
        from: Self::Anchor,
    ) -> impl Iterator<Item = (Self::Anchor, Self::LineId)> {
        batch.clone().flat_map(move |index| {
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
                (
                    self.handle.next_anchor(nails::Anchor {
                        idx: to_idx,
                        link: to_link,
                    }),
                    index,
                )
            })
        })
    }

    fn get_line(&self, index: Self::LineId) -> &Line {
        &self.lines[index]
    }

    fn get_line_mut(&mut self, index: Self::LineId) -> &mut Line {
        &mut self.lines[index]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<N> {
    #[error(transparent)]
    Nail(N),
    #[error("The minimum distance between nails must be smaller than {0}.")]
    Distancer(usize),
}
