#![allow(type_alias_bounds)]

use std::iter::FusedIterator;

use crate::*;
use board::*;
use geometry::Rect;
use math::*;
use sync::*;

pub trait Pipeline: Iterator<Item = Self::Index> {
    type Anchor: CondSend + CondSync + Clone;
    type Color: Clone;
    type Index: Clone + Into<usize>;
    type Scalar: Scalar;

    fn color_map(&self, index: &Self::Index) -> &ColorMap<Self::Anchor, Self::Color, Frac<Self>>;

    fn color_map_mut(
        &mut self,
        index: &Self::Index,
    ) -> &mut ColorMap<Self::Anchor, Self::Color, Frac<Self>>;

    fn color_maps(&self) -> &[ColorMap<Self::Anchor, Self::Color, Frac<Self>>];

    fn color_maps_mut(&mut self) -> &mut [ColorMap<Self::Anchor, Self::Color, Frac<Self>>];
}

type Frac<P: Pipeline> = <P::Scalar as Scalar>::Frac;
type Int<P: Pipeline> = <P::Scalar as Scalar>::Int;

pub struct Computation<B, P: Pipeline, D> {
    pipeline: P,
    inner: Inner<B, P, D>,
}

#[derive(Debug, thiserror::Error)]
#[error("The length of the color map buffer does not match the length of the rectangle area.")]
pub struct RectAreaError;

impl<'a, B, P: Pipeline, D> Computation<B, P, D> {
    pub fn board(&self) -> &B {
        &self.inner.board
    }

    pub fn rect(&self) -> &Rect<usize> {
        &self.inner.rect
    }

    pub fn pipeline(&self) -> &P {
        &self.pipeline
    }
}

impl<'a, B, P, D> Computation<B, P, D>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    D: decay::Fn<Frac<P>>,
{
    pub fn new<T: IntoBoard<Board = B>>(
        pipeline: P,
        into_board: T,
        rect: Rect<usize>,
        decay: D,
    ) -> Result<Self, RectAreaError> {
        let mut this = Self {
            pipeline,
            inner: Inner {
                board: into_board.into_board(cpus()),
                rect: rect,
                bufs: vec![None; cpus()],
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

impl<'a, B, P, D> Iterator for Computation<B, P, D>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    D: decay::Fn<Frac<P>>,
{
    type Item = Result<Step<P>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pipeline.next().and_then(|index| {
            let map = self.pipeline.color_map_mut(&index);
            self.inner.get_best_line(map).map(|line| {
                let segment = match B::get_segment_mut(&mut self.inner.board, line.segment_idx) {
                    Some(segment) => segment,
                    None => return Err(Error),
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

impl<'a, B, P, D> ExactSizeIterator for Computation<B, P, D>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor> + ExactSizeIterator,
    D: decay::Fn<Frac<P>>,
{
    fn len(&self) -> usize {
        self.pipeline.len()
    }
}

impl<'a, B, P, D> FusedIterator for Computation<B, P, D>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor> + FusedIterator,
    D: decay::Fn<Frac<P>>,
{
}

#[derive(Debug, thiserror::Error)]
#[error("Computation error")]
pub struct Error;

struct Inner<B, P: Pipeline, D> {
    board: B,
    decay: D,
    rect: Rect<usize>,
    bufs: Vec<Option<NextLineWeighted<P::Anchor, P::Scalar>>>,
}

impl<'a, B, P, D> Inner<B, P, D>
where
    B: Board,
    P: Pipeline<Anchor = B::Anchor>,
    D: decay::Fn<Frac<P>>,
{
    fn get_best_line(
        &mut self,
        map: &mut ColorMap<P::Anchor, P::Color, Frac<P>>,
    ) -> Option<NextLine<B::Anchor>> {
        for_each(&mut self.bufs, |i, next_line| {
            *next_line = None;
            for segment in self.board.batch(&map.anchor, i) {
                let board_segment = segment.segment();

                if board_segment.is_used() {
                    continue;
                }

                let weight = {
                    let mut count = Int::<P>::ZERO;
                    let mut weight = P::Scalar::ZERO;
                    for idx in self.rect.get_pixel_indexes_in_segment(board_segment) {
                        let delta = &map.weights[idx];
                        count = count + Int::<P>::ONE;
                        weight += <P::Scalar as Scalar>::from_frac(*delta);
                    }
                    if count > Int::<P>::ZERO {
                        weight / P::Scalar::from_int(count)
                    } else {
                        P::Scalar::ZERO
                    }
                };

                if match next_line {
                    Some(next_line) => weight > next_line.weight,
                    None => true,
                } {
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

        self.bufs
            .iter_mut()
            .filter_map(|a| a.take())
            .max_by(|a, b| a.weight.cmp(&b.weight))
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
struct NextLineWeighted<A, S> {
    next: NextLine<A>,
    weight: S,
}

impl<A: Default, S: Integer> Default for NextLineWeighted<A, S> {
    fn default() -> Self {
        Self {
            next: Default::default(),
            weight: S::ZERO,
        }
    }
}
