use std::iter::FusedIterator;

use crate::num::{Frac, Scalar, scalar_from_frac};
use crate::*;
use board::*;
use geometry::Rect;

pub struct ColorMap<A, C> {
    pub anchor: A,
    pub color: C,
    pub weights: Vec<Frac>,
}

pub trait Pipeline: Iterator<Item = Self::Index> {
    #[cfg(feature = "multithreaded")]
    type Anchor: Send + Sync + Clone;
    #[cfg(not(feature = "multithreaded"))]
    type Anchor = Clone;
    type Color: Clone;
    type Index: Clone + Into<usize>;

    fn color_map(&self, index: &Self::Index) -> &ColorMap<Self::Anchor, Self::Color>;

    fn color_map_mut(&mut self, index: &Self::Index) -> &mut ColorMap<Self::Anchor, Self::Color>;

    fn color_maps(&self) -> &[ColorMap<Self::Anchor, Self::Color>];

    fn color_maps_mut(&mut self) -> &mut [ColorMap<Self::Anchor, Self::Color>];
}

pub struct Computation<B, P: Pipeline, X: Executor> {
    pipeline: P,
    inner: Inner<B, P, X>,
}

#[derive(Debug, thiserror::Error)]
#[error("The length of the color map buffer does not match the length of the rectangle area.")]
pub struct RectAreaError;

impl<'a, B, P: Pipeline, X: Executor> Computation<B, P, X> {
    pub fn board(&self) -> &B {
        &self.inner.board
    }

    pub fn rect(&self) -> &Rect<usize> {
        &self.inner.rect
    }

    pub fn pipeline(&self) -> &P {
        &self.pipeline
    }

    /// The parallelism backend it was created with.
    pub fn par(&self) -> &X {
        &self.inner.par
    }
}

impl<'a, B, P, X> Computation<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    X: Executor,
{
    pub fn new<T: IntoBoard<Board = B>>(
        par: X,
        pipeline: P,
        into_board: T,
        rect: Rect<usize>,
        decay: decay::Decay,
    ) -> Result<Self, RectAreaError> {
        let workers = par.workers();
        let mut this = Self {
            pipeline,
            inner: Inner {
                par,
                board: into_board.into_board(workers),
                rect: rect,
                bufs: vec![None; workers],
                decay,
            },
        };
        let map_len = this.inner.rect.area();
        for map in this.pipeline.color_maps_mut() {
            if map.weights.len() != map_len {
                return Err(RectAreaError);
            }
            if let Some(line) = this.inner.get_best_line(map) {
                map.anchor = line.anchor;
            }
        }
        Ok(this)
    }
}

impl<'a, B, P, X> Iterator for Computation<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    X: Executor,
{
    type Item = Result<Step<P>, StepError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pipeline.next().and_then(|index| {
            let map = self.pipeline.color_map_mut(&index);
            self.inner.get_best_line(map).map(|line| {
                let segment = match B::get_segment_mut(&mut self.inner.board, line.segment_idx) {
                    Some(segment) => segment,
                    None => return Err(StepError),
                };
                segment.set_used();
                let anchor = core::mem::replace(&mut map.anchor, line.anchor);
                for point in self
                    .inner
                    .rect
                    .get_pixel_indexes_in_segment(segment.get_segment())
                {
                    let weight = &mut map.weights[point];
                    *weight = self.inner.decay.compute(*weight);
                }

                Ok(Step {
                    color: index,
                    anchor,
                    segment: *segment.get_segment(),
                })
            })
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pipeline.size_hint()
    }
}

impl<'a, B, P, X> ExactSizeIterator for Computation<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor> + ExactSizeIterator,
    X: Executor,
{
    fn len(&self) -> usize {
        self.pipeline.len()
    }
}

impl<'a, B, P, X> FusedIterator for Computation<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor> + FusedIterator,
    X: Executor,
{
}

#[derive(Debug, thiserror::Error)]
#[error("Computation error")]
pub struct StepError;

struct Inner<B, P: Pipeline, X: Executor> {
    par: X,
    board: B,
    decay: decay::Decay,
    rect: Rect<usize>,
    bufs: Vec<Option<NextLineWeighted<P::Anchor>>>,
}

impl<'a, B, P, X> Inner<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    X: Executor,
{
    fn get_best_line(
        &mut self,
        map: &mut ColorMap<P::Anchor, P::Color>,
    ) -> Option<NextLine<B::Anchor>> {
        let Self {
            par,
            board,
            rect,
            bufs,
            ..
        } = self;

        par.for_each(bufs, |i, next_line| {
            *next_line = None;
            for segment in board.batch(&map.anchor, i) {
                let board_segment = segment.segment();

                if board_segment.is_used() {
                    continue;
                }

                let weight = {
                    let mut count = 0u32;
                    let mut weight = Scalar::ZERO;
                    for idx in rect.get_pixel_indexes_in_segment(board_segment) {
                        weight += scalar_from_frac(map.weights[idx]);
                        count += 1;
                    }
                    if count > 0 {
                        weight / Scalar::from_num(count)
                    } else {
                        Scalar::ZERO
                    }
                };

                // Tie-break by segment index: this way the result does not depend
                // on how the candidates are distributed among threads.
                let better = match next_line {
                    Some(current) => {
                        (weight, segment.index()) > (current.weight, current.next.segment_idx)
                    }
                    None => true,
                };

                if better {
                    *next_line = Some(NextLineWeighted {
                        weight,
                        next: NextLine {
                            segment_idx: segment.index(),
                            anchor: segment.anchor(),
                        },
                    })
                }
            }
        });

        bufs.iter_mut()
            .filter_map(|a| a.take())
            .max_by(|a, b| (a.weight, a.next.segment_idx).cmp(&(b.weight, b.next.segment_idx)))
            .map(|a| a.next)
    }
}

#[derive(Debug)]
pub struct Step<P: Pipeline> {
    pub color: P::Index,
    pub anchor: P::Anchor,
    pub segment: geometry::Segment<f32>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct NextLine<A> {
    segment_idx: usize,
    anchor: A,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct NextLineWeighted<A> {
    next: NextLine<A>,
    weight: Scalar,
}

impl<A: Default> Default for NextLineWeighted<A> {
    fn default() -> Self {
        Self {
            next: Default::default(),
            weight: Scalar::ZERO,
        }
    }
}
