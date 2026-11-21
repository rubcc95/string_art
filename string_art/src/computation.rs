use std::iter::FusedIterator;

use crate::num::Frac;
#[cfg(feature = "clip")]
use crate::walk::Clip;
use crate::walk::PixelWalk;
#[cfg(feature = "cached")]
use crate::walk::{CacheTooLarge, Cached};
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

pub trait ComputationExecutor<B, P, W>:
    for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, W>>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    W: PixelWalk,
{
}

impl<T, B, P, W> ComputationExecutor<B, P, W> for T
where
    T: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, W>>,
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    W: PixelWalk,
{
}
use internal::*;
mod internal {

    use super::{Pipeline, PixelWalk};
    use crate::{Board, Frac, Scalar, Work, board::SegmentRef as _};

    pub struct ComputationWork<'a, B, P: Pipeline, W> {
        pub board: &'a B,
        pub anchor: &'a P::Anchor,
        pub weights: &'a [Frac],
        pub walk: &'a W,
    }

    impl<'a, B, P, W> Work<WorkBuffer<P>> for ComputationWork<'a, B, P, W>
    where
        B: Board,
        P: Pipeline<Anchor = B::Anchor>,
        W: PixelWalk,
    {
        fn run(&self, i: usize, next_line: &mut WorkBuffer<P>) {
            *next_line = None;
            for segment in self.board.batch(self.anchor, i) {
                let board_segment = segment.segment();
                if board_segment.is_used() {
                    continue;
                }

                let weight = self
                    .walk
                    .accumulate(segment.index(), board_segment, self.weights);

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

pub struct Computation<B, P: Pipeline, X, W: PixelWalk> {
    pipeline: P,
    inner: Inner<B, P, X, W>,
}

#[derive(Debug, thiserror::Error)]
#[error("The length of the color map buffer does not match the length of the rectangle area.")]
pub struct RectAreaError;

#[cfg(feature = "cached")]
#[derive(Debug, thiserror::Error)]
pub enum CachedError {
    #[error(transparent)]
    Area(#[from] RectAreaError),
    #[error(transparent)]
    Cache(#[from] CacheTooLarge),
}

impl<B, P: Pipeline, X, W: PixelWalk> Computation<B, P, X, W> {
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
    /// The pixel walker it was created with.
    pub fn walk(&self) -> &W {
        &self.inner.walk
    }
}

impl<B, P, X, W> Computation<B, P, X, W>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    W: PixelWalk,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, W>>,
{
    /// Builds the computation from an already-constructed board.
    ///
    /// `board` must have been produced with the same executor: its batches are
    /// sized from `par.workers()`.
    pub fn with_board(
        par: X,
        pipeline: P,
        board: B,
        rect: Rect<usize>,
        decay: decay::Decay,
        walk: W,
    ) -> Result<Self, RectAreaError> {
        let workers = par.workers();
        let mut this = Self {
            pipeline,
            inner: Inner {
                par,
                board,
                rect,
                bufs: vec![None; workers],
                decay,
                walk,
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

    /// Builds the computation with a walker derived from the constructed board.
    pub fn new_with<T: IntoBoard<Board = B>>(
        par: X,
        pipeline: P,
        into_board: T,
        rect: Rect<usize>,
        decay: decay::Decay,
        make_walk: impl FnOnce(&B) -> W,
    ) -> Result<Self, RectAreaError> {
        let workers = par.workers();
        let board = into_board.into_board(workers);
        let walk = make_walk(&board);
        Self::with_board(par, pipeline, board, rect, decay, walk)
    }
}

#[cfg(feature = "clip")]
impl<B, P, X> Computation<B, P, X, Clip>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, Clip>>,
{
    pub fn new<T: IntoBoard<Board = B>>(
        par: X,
        pipeline: P,
        into_board: T,
        rect: Rect<usize>,
        decay: decay::Decay,
    ) -> Result<Self, RectAreaError> {
        Self::new_with(par, pipeline, into_board, rect, decay, |_| Clip::new(rect))
    }
}

#[cfg(feature = "cached")]
impl<B, P, X> Computation<B, P, X, Cached>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, Cached>>,
{
    /// Builds the computation with a fully precomputed line cache.
    ///
    /// `max_bytes` is a hard budget: if the cache would not fit, this returns
    /// [`CachedError::Cache`] before allocating anything.
    pub fn new_cached<T: IntoBoard<Board = B>>(
        par: X,
        pipeline: P,
        into_board: T,
        rect: Rect<usize>,
        decay: decay::Decay,
        max_bytes: usize,
    ) -> Result<Self, CachedError> {
        let workers = par.workers();
        let board = into_board.into_board(workers);
        let walk = Cached::from_board(&board, rect, max_bytes)?;
        Ok(Self::with_board(par, pipeline, board, rect, decay, walk)?)
    }
}

impl<B, P, X, W> Iterator for Computation<B, P, X, W>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    W: PixelWalk,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, W>>,
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
                self.inner.walk.apply_decay(
                    line.segment_idx,
                    segment.get_segment(),
                    &mut map.weights,
                    self.inner.decay,
                );
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

impl<B, P, X, W> ExactSizeIterator for Computation<B, P, X, W>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor> + ExactSizeIterator,
    W: PixelWalk,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, W>>,
{
    fn len(&self) -> usize {
        self.pipeline.len()
    }
}

impl<B, P, X, W> FusedIterator for Computation<B, P, X, W>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor> + FusedIterator,
    W: PixelWalk,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, W>>,
{
}

#[derive(Debug, thiserror::Error)]
#[error("Computation error")]
pub struct StepError;

struct Inner<B, P: Pipeline, X, W> {
    par: X,
    board: B,
    decay: decay::Decay,
    rect: Rect<usize>,
    bufs: Vec<WorkBuffer<P>>,
    walk: W,
}

impl<B, P, X, W> Inner<B, P, X, W>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    W: PixelWalk,
    X: for<'a> Executor<WorkBuffer<P>, ComputationWork<'a, B, P, W>>,
{
    fn get_best_line(
        &mut self,
        map: &mut ColorMap<P::Anchor, P::Color>,
    ) -> Option<NextLine<B::Anchor>> {
        let work = ComputationWork {
            board: &self.board,
            anchor: &map.anchor,
            weights: &map.weights,
            walk: &self.walk,
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
