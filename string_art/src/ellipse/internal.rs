use super::Ellipse;
use super::SegmentRef;
use crate::board::{Board, IntoBoard, Segment, SegmentRef as SegmentRefTrait};
use crate::nails;
use crate::nails::Handle;
use derive_more::{Deref, DerefMut};
use std::fmt::{Display, Formatter};

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

#[derive(Deref, DerefMut)]
pub struct EllipseBoard<B: nails::Builder> {
    #[deref]
    #[deref_mut]
    ellipse: Ellipse<B>,
    total_combs: usize,
    per_batch: usize,
}

impl<B: nails::Builder> Display for EllipseBoard<B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    ) -> impl Iterator<Item: SegmentRefTrait<Anchor = Self::Anchor>> + 'a {
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
                    // Si esto revienta, el culpable es `index_of` o el contrato de
                    // `Links` (orden de iteración vs `index_of`), no quien llama.
                    segment: &self.segments[index],
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
