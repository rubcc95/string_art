use crate::geometry::Segment;
use derive_more::Deref;

#[cfg(feature = "rayon")]
mod multi_thread {
    use super::Board;

    pub trait ValidBoard
    where
        Self: Sync,
        Self: Board<Batch: Send + Sync>,
        Self: Board<LineId: Send + Sync>,
        Self: Board<Anchor: Send + Sync>,
    {
    }

    impl<T: Board> ValidBoard for T
    where
        T: Sync,
        T: Board<Batch: Send + Sync>,
        T: Board<LineId: Send + Sync>,
        T: Board<Anchor: Send + Sync>,
    {
    }
}

#[cfg(not(feature = "rayon"))]
pub use crate::Board as ValidBoard;
#[cfg(feature = "rayon")]
pub use multi_thread::ValidBoard;

pub trait Board {
    type Batch;
    type Anchor: Default + Copy;
    type LineId: Copy;

    fn get_batches(&self, cpus: usize) -> impl Iterator<Item = Self::Batch>;

    fn get_indexes(
        &self,
        batch: &Self::Batch,
        id: Self::Anchor,
    ) -> impl Iterator<Item = (Self::Anchor, Self::LineId)>;

    fn get_line(&self, index: Self::LineId) -> &Line;

    fn get_line_mut(&mut self, index: Self::LineId) -> &mut Line;

    //fn get_anchor(&self, id: Self::AnchorId) -> Self::Anchor;
}

impl<B: Board> Board for &mut B {
    type Batch = B::Batch;
    type Anchor = B::Anchor;
    type LineId = B::LineId;

    fn get_batches(&self, cpus: usize) -> impl Iterator<Item = Self::Batch> {
        B::get_batches(*self, cpus)
    }

    fn get_indexes(
        &self,
        batch: &Self::Batch,
        id: Self::Anchor,
    ) -> impl Iterator<Item = (Self::Anchor, Self::LineId)> {
        B::get_indexes(*self, batch, id)
    }

    fn get_line(&self, index: Self::LineId) -> &Line {
        B::get_line(*self, index)
    }

    fn get_line_mut(&mut self, index: Self::LineId) -> &mut Line {
        B::get_line_mut(*self, index)
    }

    // fn get_anchor(&self, id: Self::AnchorId) -> Self::Anchor {
    //     B::get_anchor(*self, id)
    // }
}

#[derive(Clone, Debug, Deref)]
pub struct Line {
    #[deref]
    line: Segment<f32>,
    is_used: bool,
}

impl From<Segment<f32>> for Line {
    fn from(segment: Segment<f32>) -> Self {
        Self {
            line: segment,
            is_used: false,
        }
    }
}

impl Line {
    pub fn new(segment: Segment<f32>) -> Self {
        Self {
            line: segment,
            is_used: false,
        }
    }

    pub fn segment(&self) -> &Segment<f32> {
        &self.line
    }

    pub fn is_used(&self) -> bool {
        self.is_used
    }

    pub fn set_used(&mut self) {
        debug_assert_eq!(self.is_used, false);
        self.is_used = true;
    }
}
