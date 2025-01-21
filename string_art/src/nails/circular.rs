use rand::{distributions::Standard, prelude::Distribution, Rng};

use crate::{
    geometry::{
        circle, Circle, Point, Segment
    },
    nails::{Builder, Handle, Links},
    Float,
};

#[derive(Clone, Copy)]
pub struct Circular<T = f32>(T);

impl<T> Circular<T> {
    pub fn new(radius: T) -> Self {
        Self(radius)
    }
}

impl<T: Float> Builder for Circular<T> {
    type Scalar = T;
    type Handle = Self;
    type Nail = Point<T>;

    fn build_nail(&self, point: Point<T>, _: T) -> Self::Nail {
        point
    }

    fn build_handle(self) -> Self::Handle {
        self
    }
    
}

impl<T: Float> Handle for Circular<T> {
    type Scalar = T;

    type Nail = Point<T>;

    type Links = CircularLinks;

    type Link = Direction;

    type Error = Error;

    const LINKS: Self::Links = CircularLinks;

    fn get_segment(
        self,
        start: (&Self::Nail, <Self::Links as IntoIterator>::Item),
        end: (&Self::Nail, <Self::Links as IntoIterator>::Item),
    ) -> Result<Segment<T>, Self::Error> {
        Circle {
            center: *start.0,
            radius: self.0,
        }
        .tangent(
            start.1.0,
            Circle {
                center: *end.0,
                radius: self.0,
            },
            end.1.0,
        ).ok_or(Error)
    }

    fn get_next_link(self, prev_link: Direction) -> Direction {
        prev_link
    }
    
    fn draw_svg(self, nail: Self::Nail) -> impl Into<Box<dyn svg::Node>> {
        svg::node::element::Circle::new()
        .set("cx", nail.x)      // Coordenada X del centro
        .set("cy", nail.y)      // Coordenada Y del centro
        .set("r", self.0)        // Radio
        .set("fill", "black")   // Color de relleno
    }
    
    
}

pub struct CircularLinks;

unsafe impl Links for CircularLinks {
    const LEN: usize = 2;
    
    type Link = Direction;
}

impl IntoIterator for CircularLinks {
    type Item = Direction;

    type IntoIter = core::array::IntoIter<Direction, 2>;

    fn into_iter(self) -> Self::IntoIter {
        Direction::ALL.into_iter()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Direction(circle::Direction);



impl core::fmt::Display for Direction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            // Since instructions are reversed to the build process, we swap values
            circle::Direction::ClockWise => write!(f, "CounterClockWise"),
            circle::Direction::CounterClockWise => write!(f, "ClockWise"),
        }
    }
}

impl Default for Direction{
    fn default() -> Self {
        Self(circle::Direction::ClockWise)
    }
}

impl Direction {
    const ALL: [Self; 2] = [
        Self(circle::Direction::ClockWise),
        Self(circle::Direction::CounterClockWise),
    ];
}

impl From<Direction> for usize {
    fn from(direction: Direction) -> Self {
        direction.0 as usize
    }
}

impl Distribution<Direction> for Standard{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        Direction(unsafe { core::mem::transmute(rng.gen_range::<u8,_>(0..2)) })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The nails are overlapping")]
pub struct Error;