use crate::{
    Board, Pipeline, PipelineLayer, ValidPipelineLayer, ValidWeightMap, WeightMap,
    board::ValidBoard, geometry::Segment, math::*,
};
use log::info;
use num_traits::SaturatingSub;

#[cfg(feature = "rayon")]
use num_cpus::get as cpus;
use string_art_geometry::Rect;

#[cfg(not(feature = "rayon"))]
pub const fn cpus() -> usize {
    1
}

pub struct Computation<B: Board, P: Pipeline<Weight: Frac>> {
    runtime: P::Runtime<B::Anchor>,
    inner: BatchedBoard<B, P::Weight>,
    decay: P::Weight,
}

impl<'a, B: Board, P: Pipeline<Weight: Frac>> Computation<B, P> {
    pub fn board(&self) -> &B {
        &self.inner.board
    }

    pub fn runtime(&self) -> &P::Runtime<B::Anchor> {
        &self.runtime
    }
}

impl<B, P> Computation<B, P>
where
    B: ValidBoard,
    for<'a> P: Pipeline<Weight: Frac, Layer<'a, B::Anchor>: ValidPipelineLayer>,
{
    pub fn new(pipeline: P, board: B, decay: P::Weight) -> Self {
        env_logger::init();
        let mut batched_board = BatchedBoard {
            bufs: board.get_batches(cpus()).map(BatchBuffer::new).collect(),
            board,
            //rect: rect,
        };
        Self {
            runtime: pipeline.init(&mut batched_board),
            inner: batched_board,
            decay,
        }
    }

    pub fn rect(&self) -> Rect<u32> {
        P::rect(&self.runtime)
    }
}

impl<B, P> Iterator for Computation<B, P>
where
    B: ValidBoard,
    for<'a> P: Pipeline<Weight: Frac, Layer<'a, B::Anchor>: ValidPipelineLayer>,
{
    type Item = Step<P::MapId, B::Anchor>;

    fn next(&mut self) -> Option<Self::Item> {
        P::next(&mut self.runtime).and_then(|mut map| {
            let anchor = *map.anchor();
            self.inner.get_best_line(&map, anchor).map(|step_id| {
                let line = self.inner.board.get_line_mut(step_id.line_id);
                line.set_used();
                let anchor = *map.anchor();
                map.set_anchor(step_id.anchor);
                let segment = line.segment();
                for point in segment.floor().as_::<isize>().bresenham() {
                    if let Some(point) = point.cast::<usize>() {
                        if let Some(pixel) = map.get_mut(point) {
                            *pixel = (*pixel).saturating_sub(&self.decay);
                        }
                    }
                }

                Step {
                    layer: map.id(),
                    anchor,
                    segment: *segment,
                }
            })
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Computation error")]
pub struct Error;

pub struct BatchedBoard<B: Board, F: Frac> {
    board: B,
    bufs: Vec<BatchBuffer<B, F::Fixed>>,
}

impl<'a, B, F> BatchedBoard<B, F>
where
    B: ValidBoard,
    F: Frac,
{
    pub fn get_best_line(
        &mut self,
        map: &impl ValidWeightMap<Weight = F>,
        anchor: B::Anchor,
    ) -> Option<StepId<B::LineId, B::Anchor>> {
        info!("Holita llamando a get_best_line");
        #[cfg(not(feature = "rayon"))]
        for buffer in &mut self.bufs {
            buffer.get_best_line(&self.board, map, anchor);
        }

        #[cfg(feature = "rayon")]
        {
            use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

            self.bufs
                .par_iter_mut()
                .for_each(|buffer| buffer.get_best_line(&self.board, map, anchor));
        }

        let mut bufs = self.bufs.iter();

        bufs.next().and_then(|mut best| {
            for buf in bufs {
                if buf.weight > best.weight {
                    best = buf;
                }
            }
            best.step
        })
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Step<L, A> {
    pub layer: L,
    pub anchor: A,
    pub segment: Segment<f32>,
}

#[derive(Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StepId<L, A> {
    pub line_id: L,
    pub anchor: A,
}

#[derive(Clone)]
struct BatchBuffer<B: Board, F> {
    batch: B::Batch,
    step: Option<StepId<B::LineId, B::Anchor>>,
    weight: F,
}

impl<B: Board, F: Scalar> BatchBuffer<B, F> {
    fn new(batch: B::Batch) -> Self {
        Self {
            batch,
            step: None,
            weight: F::ZERO,
        }
    }
}

impl<B: Board, F: Fixed> BatchBuffer<B, F> {
    fn get_best_line(
        &mut self,
        board: &B,
        map: &impl WeightMap<Weight = F::Frac>,
        anchor: B::Anchor,
    ) {
        self.step = None;
        self.weight = F::ZERO;

        for (anchor, line_id) in board.get_indexes(&self.batch, anchor) {
            let line = board.get_line(line_id);

            if line.is_used() {
                continue;
            }

            let segment = line.segment();

            let weight = {
                let mut count = F::Int::ZERO;
                let mut weight = F::ZERO;
                for point in segment.floor().as_::<isize>().bresenham() {
                    if let Some(point) = point.cast::<usize>() {
                        if let Some(&delta) = map.get(point) {
                            count = count + F::Int::ONE;
                            weight += F::from_frac(delta);
                        }
                    }
                }
                if count > F::Int::ZERO {
                    weight / F::from_int(count)
                } else {
                    F::ZERO
                }
            };

            if weight > self.weight {
                self.weight = weight;
                self.step = Some(StepId { line_id, anchor });
            }
        }
    }
}
