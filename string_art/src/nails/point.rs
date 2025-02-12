
use crate::{geometry::{Point as Pt, Segment}, Float};

use super::{Builder, Handle, Links};

#[derive(Clone, Copy)]
pub struct Point<T>(core::marker::PhantomData<T>);


impl<T> Point<T>{
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T: Float> Builder for Point<T> {
    type Scalar = T;
    type Handle = Self;
    type Nail = Pt<T>;

    fn build_nail(&self, point: Pt<T>, _: T) -> Self::Nail {
        point
    }

    fn build_handle(self) -> Self::Handle {
        self
    }

    fn offset(&self) -> Self::Scalar {
        T::ZERO        
    }
    
}

impl<T: Float> Handle for Point<T> {
    type Scalar = T;

    type Nail = Pt<T>;

    type Links = PointLink;

    type Link = usize;

    type Error = Error;

    const LINKS: Self::Links = PointLink;

    fn get_segment(
        self,
        start: (&Self::Nail, <Self::Links as IntoIterator>::Item),
        end: (&Self::Nail, <Self::Links as IntoIterator>::Item),
    ) -> Result<Segment<T>, Self::Error> {
        Ok(Segment::new(*start.0, *end.0))        
    }

    fn get_next_link(self, _: usize) -> usize {
        0
    }
    
    fn draw_svg(self, nail: Self::Nail) -> impl Into<Box<dyn svg::Node>> {
        svg::node::element::Circle::new()
        .set("cx", nail.x)      
        .set("cy", nail.y)      
        .set("r", 0.01)     
        .set("fill", "black")
    }
    
    
}

pub struct PointLink;

unsafe impl Links for PointLink {
    const LEN: usize = 1;
    
    type Link = usize;
}

impl IntoIterator for PointLink {
    type Item = usize;

    type IntoIter = core::iter::Once<usize>;

    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(0)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Point error")]
pub struct Error;