use std::iter::FusedIterator;

use crate::num::Frac;
use crate::*;
use board::*;
use geometry::Rect;

pub struct ColorMap<A, C> {
    pub anchor: A,
    pub color: C,
    pub weights: Vec<Frac>,
}

pub trait Pipeline: Iterator<Item = Self::Index> {
    type Anchor: Clone;
    type Color: Clone;
    type Index: Clone + Into<usize>;

    fn color_map(&self, index: &Self::Index) -> &ColorMap<Self::Anchor, Self::Color>;
    fn color_map_mut(&mut self, index: &Self::Index) -> &mut ColorMap<Self::Anchor, Self::Color>;
    fn color_maps(&self) -> &[ColorMap<Self::Anchor, Self::Color>];
    fn color_maps_mut(&mut self) -> &mut [ColorMap<Self::Anchor, Self::Color>];
}

pub trait ComputationExecutor<B, P>:
    for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P>>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
{
}

impl<T, B, P> ComputationExecutor<B, P> for T
where
    T: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P>>,
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
{
}
use internal::*;
mod internal {

    use super::Pipeline;
    use crate::{
        Board, Frac, Scalar, Work, board::SegmentRef as _, geometry::Rect, scalar_from_frac,
    };

    pub struct ComputationWork<'a, B, P: Pipeline> {
        pub board: &'a B,
        pub rect: &'a Rect<usize>,
        pub anchor: &'a P::Anchor,
        pub weights: &'a [Frac],
    }

    impl<'a, B, P> Work<WorkBuffer<P>> for ComputationWork<'a, B, P>
    where
        B: Board,
        P: Pipeline<Anchor = B::Anchor>,
    {
        fn run(&self, i: usize, next_line: &mut WorkBuffer<P>) {
            *next_line = None;
            for segment in self.board.batch(self.anchor, i) {
                let board_segment = segment.segment();
                if board_segment.is_used() {
                    continue;
                }

                let weight = {
                    let mut count = 0u32;
                    let mut weight = Scalar::ZERO;
                    for idx in self.rect.get_pixel_indexes_in_segment(board_segment) {
                        weight += scalar_from_frac(self.weights[idx]);
                        count += 1;
                    }
                    if count > 0 {
                        weight / Scalar::from_num(count)
                    } else {
                        Scalar::ZERO
                    }
                };

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
                    });
                }
            }
        }
    }

    pub type WorkBuffer<P> = Option<NextLineWeighted<<P as Pipeline>::Anchor>>;

    #[derive(Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]

    pub struct NextLineWeighted<A> {
        pub next: NextLine<A>,
        pub weight: Scalar,
    }

    #[derive(Clone, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct NextLine<A> {
        pub segment_idx: usize,
        pub anchor: A,
    }

    impl<A: Default> Default for NextLineWeighted<A> {
        fn default() -> Self {
            Self {
                next: Default::default(),
                weight: Scalar::ZERO,
            }
        }
    }
}

pub struct Computation<B, P: Pipeline, X> {
    pipeline: P,
    inner: Inner<B, P, X>,
}

#[derive(Debug, thiserror::Error)]
#[error("The length of the color map buffer does not match the length of the rectangle area.")]
pub struct RectAreaError;

impl<B, P, X> Computation<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P>>,
{
    pub fn board(&self) -> &B {
        &self.inner.board
    }
    pub fn rect(&self) -> &Rect<usize> {
        &self.inner.rect
    }
    pub fn pipeline(&self) -> &P {
        &self.pipeline
    }
    /// The execution backend it was created with.
    pub fn par(&self) -> &X {
        &self.inner.par
    }

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
                rect,
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

impl<B, P, X> Iterator for Computation<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P>>,
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

impl<B, P, X> ExactSizeIterator for Computation<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor> + ExactSizeIterator,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P>>,
{
    fn len(&self) -> usize {
        self.pipeline.len()
    }
}

impl<B, P, X> FusedIterator for Computation<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor> + FusedIterator,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P>>,
{
}

#[derive(Debug, thiserror::Error)]
#[error("Computation error")]
pub struct StepError;

struct Inner<B, P: Pipeline, X> {
    par: X,
    board: B,
    decay: decay::Decay,
    rect: Rect<usize>,
    bufs: Vec<WorkBuffer<P>>,
}

impl<B, P, X> Inner<B, P, X>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P>>,
{
    fn get_best_line(
        &mut self,
        map: &mut ColorMap<P::Anchor, P::Color>,
    ) -> Option<NextLine<B::Anchor>> {
        let work = ComputationWork {
            board: &self.board,
            rect: &self.rect,
            anchor: &map.anchor,
            weights: &map.weights,
        };
        self.par.for_each(&mut self.bufs, work);
        self.bufs
            .iter_mut()
            .filter_map(|item| item.take())
            .max_by(|a, b| (a.weight, a.next.segment_idx).cmp(&(b.weight, b.next.segment_idx)))
            .map(|item| item.next)
    }
}

#[derive(Debug)]
pub struct Step<P: Pipeline> {
    pub color: P::Index,
    pub anchor: P::Anchor,
    pub segment: geometry::Segment<f32>,
}
