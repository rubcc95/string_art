use string_art_geometry::Point;
use string_art_math::Frac;
//use string_art_sync::*;

use crate::{board::ValidBoard, computation::BatchedBoard};

pub trait Pipeline {
    type MapId;
    type Runtime<A>;
    type Weight: Frac;
    type Layer<'a, A>: PipelineLayer<Weight = Self::Weight, AnchorId = A, MapId = Self::MapId>
    where
        Self: 'a,
        A: 'a;

    fn init<B: ValidBoard>(
        self,
        batched: &mut BatchedBoard<B, Self::Weight>,
    ) -> Self::Runtime<B::Anchor>;

    fn next<A>(runtime: &mut Self::Runtime<A>) -> Option<Self::Layer<'_, A>>;
}

pub trait PipelineLayer: WeightMap {
    type AnchorId;
    type MapId;

    fn id(&self) -> Self::MapId;
    fn anchor(&self) -> &Self::AnchorId;
    fn set_anchor(&mut self, id: Self::AnchorId);
}

pub trait WeightMap {
    type Weight;

    fn get(&self, pixel: Point<usize>) -> Option<&Self::Weight>;
    fn get_mut(&mut self, pixel: Point<usize>) -> Option<&mut Self::Weight>;
}

#[cfg(feature = "rayon")]
mod multi_thread {
    use super::{PipelineLayer, WeightMap};

    pub trait ValidPipelineLayer: ValidWeightMap + PipelineLayer {}

    impl<T: ValidWeightMap + PipelineLayer> ValidPipelineLayer for T {}

    pub trait ValidWeightMap: Sync + WeightMap {}

    impl<T: Sync + WeightMap> ValidWeightMap for T {}
}
#[cfg(feature = "rayon")]
pub use multi_thread::*;

#[cfg(not(feature = "rayon"))]
mod single_thread {
    pub use super::PipelineLayer as ValidPipelineLayer;
    pub use super::WeightMap as ValidWeightMap;
}
#[cfg(not(feature = "rayon"))]
pub use single_thread::*;
