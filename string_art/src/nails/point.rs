use crate::geometry;
use crate::nails;

#[derive(Clone, Copy)]
pub struct Point;

impl nails::Builder for Point {
    type Nail = geometry::Point<f32>;

    type Links = [Link; 1];

    type Link = Link;

    type Handle = Self;

    type Error = core::convert::Infallible;

    const LINKS: Self::Links = [Link];

    fn create_nail(&self, position: geometry::Point<f32>, _: f32) -> Self::Nail {
        position
    }

    fn create_segment(
        &self,
        start: (&Self::Nail, Self::Link),
        end: (&Self::Nail, Self::Link),
    ) -> Result<geometry::Segment<f32>, Self::Error> {
        Ok(geometry::Segment::new(*start.0, *end.0))
    }

    fn anchor_builder(self) -> Self::Handle {
        self
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Link;

impl std::fmt::Display for Link {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Into<usize> for Link {
    fn into(self) -> usize {
        0
    }
}

impl nails::Handle for Point {
    type Nail = geometry::Point<f32>;
    type Link = Link;

    fn next_anchor(&self, anchor: nails::Anchor<Link>) -> nails::Anchor<Link> {
        anchor
    }

    fn index_of(&self, _: &Self::Link) -> usize {
        0
    }

    fn reversed(&self, anchor: nails::Anchor<Self::Link>) -> nails::Anchor<Self::Link> {
        anchor
    }
}
