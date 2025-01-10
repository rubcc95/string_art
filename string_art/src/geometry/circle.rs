use std::fmt::Debug;

use image::GenericImage;

use crate::Float;

use super::{Point, Segment};

#[derive(Clone, Copy, Debug)]
pub struct Circle<T> {
    pub center: Point<T>,
    pub radius: T,
}

impl<T: Float> Circle<T> {
    pub fn tangent(
        self,
        dir: Direction,
        other: Self,
        other_dir: Direction,
    ) -> Option<Segment<T>> {
        if dir == other_dir {
            self.outer_tangent(other, dir)
        } else {
            self.inner_tangent(other, dir)
        }
    }

    pub fn inner_tangent(self, other: Self, dir: Direction) -> Option<Segment<T>> {
        let dx = other.center.x - self.center.x;
        let dy = other.center.y - self.center.y;
        let dist = num_traits::Float::sqrt(dx * dx + dy * dy);
        if dist <= (self.radius + other.radius) {
            return None;
        }

        let angle1 = dy.atan2(dx);
        let angle2 = ((self.radius + other.radius) / dist).acos();
        let (x_a, y_a) = if dir == Direction::ClockWise {
            ((angle1 + angle2).cos(),(angle1 + angle2).sin())
        } else {
            ((angle1 - angle2).cos(),(angle1 - angle2).sin())
        };
        Some(Segment {
            start: Point {
                x: self.center.x + self.radius * x_a,
                y: self.center.y + self.radius * y_a,
            },
            end: Point {
                x: other.center.x - other.radius * x_a,
                y: other.center.y - other.radius * y_a,
            },
        })
    }

    pub fn outer_tangent(self, other: Self, dir: Direction) -> Option<Segment<T>> {
        let dx = other.center.x - self.center.x;
        let dy = other.center.y - self.center.y;
        let dist = num_traits::Float::sqrt(dx * dx + dy * dy);
        if dist <= (self.radius - other.radius).abs() {
            return None;
        }

        let angle1 = dy.atan2(dx);
        let angle2 = ((self.radius - other.radius) / dist).acos();
        let (x_a, y_a) = if dir == Direction::ClockWise {
            ((angle1 + angle2).cos(), (angle1 + angle2).sin())
        } else {
            ((angle1 - angle2).cos(), (angle1 - angle2).sin())
        };
        Some(Segment {
            start: Point {
                x: self.center.x + self.radius * x_a,
                y: self.center.y + self.radius * y_a,
            },
            end: Point {
                x: other.center.x + other.radius * x_a,
                y: other.center.y + other.radius * y_a,
            },
        })
    }

    pub fn draw<I: GenericImage>(self, image: &mut I, pixel: I::Pixel) {
        let mut y = -self.radius;
        while y <= self.radius {
            let x = num_traits::Float::sqrt(self.radius * self.radius - y * y);
            let x0 = self.center.x - x;
            let x1 = self.center.x + x;
            let y0 = self.center.y + y;
            image.put_pixel(x0.to_u32().unwrap(), y0.to_u32().unwrap(), pixel);
            image.put_pixel(x1.to_u32().unwrap(), y0.to_u32().unwrap(), pixel);
            y = y + T::ONE;
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    ClockWise,
    CounterClockWise,
}

impl Direction {
    pub const ALL: [Direction; 2] = [Direction::ClockWise, Direction::CounterClockWise];
}


