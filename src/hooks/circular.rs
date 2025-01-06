use crate::{
    geometry::{
        circle, Circle, Point, Segment
    },
    hooks::{Builder, Handle, Links},
    Float,
};

#[derive(Clone, Copy)]
pub struct Circular<T = f32>(T);

impl<T> Circular<T> {
    pub fn new(radius: T) -> Self {
        Self(radius)
    }
}

impl<T> Builder<T> for Circular<T> {
    type Handle = Self;

    type Hook = Point<T>;

    fn build_hook(&self, point: Point<T>, _: T) -> Self::Hook {
        point
    }

    fn build_handle(self) -> Self::Handle {
        self
    }
}

impl<T: Float> Handle<T> for Circular<T> {
    type Hook = Point<T>;

    type Links = CircularLinks;

    const LINKS: Self::Links = CircularLinks;

    fn get_segment(
        &self,
        start: (&Self::Hook, <Self::Links as IntoIterator>::Item),
        end: (&Self::Hook, <Self::Links as IntoIterator>::Item),
    ) -> Segment<T> {
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
        )
        .expect(
            "The nails are too close together in this configuration. \
            Increase the size of your board, or decrease the hook radius.",
        )
    }

    fn get_next_link(&self, prev_link: Direction) -> Direction {
        prev_link
    }
}

pub struct CircularLinks;

unsafe impl Links for CircularLinks {
    const LEN: usize = 2;
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
            circle::Direction::ClockWise => write!(f, "ClockWise"),
            circle::Direction::CounterClockWise => write!(f, "CounterClockWise"),
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
