use crate::nails;
use crate::geometry;


#[derive(Clone, Copy)]
pub struct Point;

impl nails::Builder for Point {
    type Nail = geometry::Point<f32>;

    type Links = Links;

    type Link = Link;

    type Handle = Self;

    type Error = Error;

    const LINKS: Self::Links = Links;

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

pub struct Links;

unsafe impl nails::Links for Links {
    const LEN: usize = 1;

    type Link = Link;
}

impl IntoIterator for Links {
    type Item = Link;

    type IntoIter = core::iter::Once<Link>;

    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(Link)
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

impl TryFrom<usize> for Link {
    type Error = FromIndexError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value{
            0 => Ok(Self),
            val => Err(FromIndexError(val))
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Index {0} is out of range")]
pub struct FromIndexError(usize);

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
    // #[cfg(feature = "svg")]
    // fn draw(&self, _: &Self::Nail, d: &mut svg::Document) {
    //     let a = d.get_children();
    // }
}

#[derive(Debug, thiserror::Error)]
#[error("Point error")]
pub struct Error;
