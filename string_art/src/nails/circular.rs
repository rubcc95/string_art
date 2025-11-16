use crate::geometry::*;
use crate::nails;
use derive_more::*;

#[derive(Clone, Copy, Debug, Deref, DerefMut, From)]
pub struct UniformCircular(pub f32);

impl nails::Builder for UniformCircular {
    type Nail = Point<f32>;

    type Links = Links;

    type Handle = Self;

    type Link = Direction;

    type Error = Error;

    const LINKS: Self::Links = Links;

    fn create_nail(&self, position: Point<f32>, _: f32) -> Self::Nail {
        position
    }

    fn create_segment(
        &self,
        start: (&Self::Nail, Self::Link),
        end: (&Self::Nail, Self::Link),
    ) -> Result<Segment<f32>, Self::Error> {
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
        .ok_or(Error)
    }

    fn anchor_builder(self) -> Self::Handle {
        self
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Links;

unsafe impl nails::Links for Links {
    const LEN: usize = 2;

    type Link = Direction;
}

impl IntoIterator for Links {
    type Item = Direction;

    type IntoIter = core::array::IntoIter<Direction, 2>;

    fn into_iter(self) -> Self::IntoIter {
        Direction::ALL.into_iter()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut, From, Debug)]
pub struct Direction(circle::Direction);

impl Direction {
    pub const CLOCK_WISE: Self = Direction(circle::Direction::ClockWise);

    pub const COUNTER_CLOCK_WISE: Self = Direction(circle::Direction::CounterClockWise);
}

impl core::fmt::Display for Direction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            // Since instructions are reversed to the build process, we swap values
            circle::Direction::ClockWise => write!(f, "CounterClockWise"),
            circle::Direction::CounterClockWise => write!(f, "ClockWise"),
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self(circle::Direction::ClockWise)
    }
}

impl Direction {
    const ALL: [Self; 2] = [
        Self(circle::Direction::ClockWise),
        Self(circle::Direction::CounterClockWise),
    ];

    pub fn reversed(self) -> Self{
        match self.0 {
            circle::Direction::ClockWise => Self::COUNTER_CLOCK_WISE,
            circle::Direction::CounterClockWise => Self::CLOCK_WISE,
        }
    }
}

impl From<Direction> for usize {
    fn from(direction: Direction) -> Self {
        direction.0 as usize
    }
}

impl TryFrom<usize> for Direction {
    type Error = FromIndexError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self(circle::Direction::ClockWise)),
            1 => Ok(Self(circle::Direction::CounterClockWise)),
            val => Err(FromIndexError(val)),
        }
    }
}
#[derive(Debug, thiserror::Error)]
#[error("Index {0} is out of range")]
pub struct FromIndexError(usize);

impl nails::Handle for UniformCircular {
    type Nail = Point<f32>;
    type Link = Direction;

    fn next_anchor(&self, anchor: nails::Anchor<Direction>) -> nails::Anchor<Direction> {
        anchor
    }

    fn index_of(&self, link: &Self::Link) -> usize {
        match link.0 {
            circle::Direction::ClockWise => 0,
            circle::Direction::CounterClockWise => 1,
        }
    }
    
    fn reversed(&self, mut anchor: nails::Anchor<Self::Link>) -> nails::Anchor<Self::Link> {
        anchor.link = anchor.link.reversed();
        anchor
    }
}

#[derive(Debug, thiserror::Error)]
#[error("The nails are overlapping")]
pub struct Error;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{geometry::Point, nails::Builder};

    #[test]
    fn test_circular_tangents_are_correct() {
        let radius = 1.0;
        let circular = UniformCircular(radius);

        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(4.0, 0.0);

        let clock_to_clock = circular
            .create_segment((&p1, Direction::CLOCK_WISE), (&p2, Direction::CLOCK_WISE))
            .expect("Should create CW tangent");

        assert!(clock_to_clock.start.y > 0.0);
        assert!(clock_to_clock.end.y > 0.0);

        let counter_to_counter = circular
            .create_segment(
                (&p1, Direction::COUNTER_CLOCK_WISE),
                (&p2, Direction::COUNTER_CLOCK_WISE),
            )
            .expect("Should create CCW tangent");

        assert!(counter_to_counter.start.y < 0.0);
        assert!(counter_to_counter.end.y < 0.0);

        let clock_to_counter = circular
            .create_segment(
                (&p1, Direction::CLOCK_WISE),
                (&p2, Direction::COUNTER_CLOCK_WISE),
            )
            .expect("Should create CW tangent");

        assert!(clock_to_counter.start.y > 0.0);
        assert!(clock_to_counter.end.y < 0.0);

        let counter_to_clock = circular
            .create_segment(
                (&p1, Direction::COUNTER_CLOCK_WISE),
                (&p2, Direction::CLOCK_WISE),
            )
            .expect("Should create CCW tangent");

        assert!(counter_to_clock.start.y < 0.0);
        assert!(counter_to_clock.end.y > 0.0);

        let clock_to_clock = circular
            .create_segment((&p2, Direction::CLOCK_WISE), (&p1, Direction::CLOCK_WISE))
            .expect("Should create CW tangent");

        assert!(clock_to_clock.start.y < 0.0);
        assert!(clock_to_clock.end.y < 0.0);

        let counter_to_counter = circular
            .create_segment(
                (&p2, Direction::COUNTER_CLOCK_WISE),
                (&p1, Direction::COUNTER_CLOCK_WISE),
            )
            .expect("Should create CCW tangent");

        assert!(counter_to_counter.start.y > 0.0);
        assert!(counter_to_counter.end.y > 0.0);

        let clock_to_counter = circular
            .create_segment(
                (&p2, Direction::COUNTER_CLOCK_WISE),
                (&p1, Direction::CLOCK_WISE),
            )
            .expect("Should create CW tangent");

        assert!(clock_to_counter.start.y > 0.0);
        assert!(clock_to_counter.end.y < 0.0);

        let counter_to_clock = circular
            .create_segment(
                (&p2, Direction::CLOCK_WISE),
                (&p1, Direction::COUNTER_CLOCK_WISE),
            )
            .expect("Should create CCW tangent");

        assert!(counter_to_clock.start.y < 0.0);
        assert!(counter_to_clock.end.y > 0.0);
    }
}
