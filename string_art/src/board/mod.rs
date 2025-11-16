
use derive_more::Deref;
use crate::sync::*;

type Seg = crate::geometry::Segment<f32>;

pub trait IntoBoard{
    type Board: Board;

    fn into_board(self, cpus: usize) -> Self::Board;
}

pub trait Board : CondSync{
    type Anchor: Clone + Default + CondSend + CondSync;    

    fn batch<'a>(
        &'a self,
        anchor: &'a Self::Anchor,
        index: usize,
    ) -> impl Iterator<Item: SegmentRef<Anchor = Self::Anchor>> + 'a;

    fn get_segment(&self, idx: usize) -> Option<&Segment>;

    fn get_segment_mut(&mut self, idx: usize) -> Option<&mut Segment>;
}

pub trait SegmentRef {
    type Anchor;

    fn segment(&self) -> &Segment;

    fn index(&self) -> usize;

    fn anchor(&self) -> Self::Anchor;
}

pub trait Batched {
    type Anchor;
    type Ref: SegmentRef<Anchor = Self::Anchor>;

    fn segments(&self) -> impl Iterator<Item = Self::Ref>;
}

#[derive(Clone, Debug, Deref)]
pub struct Segment {
    #[deref]
    segment: Seg,
    is_used: bool,
}

impl From<Seg> for Segment {
    fn from(segment: Seg) -> Self {
        Self {
            segment,
            is_used: false,
        }
    }
}

impl Segment {
    pub fn new(segment: Seg) -> Self {
        Self {
            segment,
            is_used: false,
        }
    }

    pub fn get_segment(&self) -> &Seg {
        &self.segment
    }

    pub fn is_used(&self) -> bool {
        self.is_used
    }

    pub fn set_used(&mut self) {
        debug_assert_eq!(self.is_used, false);
        self.is_used = true;
    }
}
