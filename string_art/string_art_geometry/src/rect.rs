use crate::{Point, Segment};
use num_traits::*;
use std::ops::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq,)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect<T> {
    pub height: T,
    pub width: T,
}

impl<T> Rect<T> {
    pub fn new(height: T, width: T) -> Self {
        Self { height, width }
    }
}

impl<T> From<Point<T>> for Rect<T> {
    fn from(point: Point<T>) -> Self {
        Self {
            height: point.y,
            width: point.x,
        }
    }
}

impl<T> From<Rect<T>> for Point<T> {
    fn from(value: Rect<T>) -> Self {
        Self {
            x: value.width,
            y: value.height,
        }
    }
}

impl<S: NumCast> Rect<S> {
    pub fn cast<I: NumCast>(self) -> Option<Rect<I>> {
        cast(self.width)
            .and_then(|width| cast(self.height).map(|height| Rect { width, height }))
    }
}

impl<S> Rect<S> {
    pub fn as_<I: Copy + 'static>(self) -> Rect<I>
    where
        S: AsPrimitive<I>,
    {
        Rect {
            width: self.width.as_(),
            height: self.height.as_(),
        }
    }
}

impl<T: Mul + Clone> Rect<T> {
    pub fn area(&self) -> T::Output {
        self.width.clone() * self.height.clone()
    }
}

impl<T: NumCast + Unsigned + PartialOrd + Copy> Rect<T> {
    pub fn index_of(&self, point: Point<T>) -> Option<T> {
        if point.x < self.width && point.y < self.height {
            Some(point.y * self.width + point.x)
        } else {
            None
        }
    }

    pub unsafe fn index_of_unchecked(&self, point: Point<T>) -> T {
        point.y * self.width + point.x
    }
}

impl Rect<usize> {
    /// Visits the flat indexes of the pixels crossed by `segment`.
    ///
    /// Uses the integer Bresenham line: the start is included, the end is
    /// excluded, and out-of-bounds pixels are dropped. When the segment's
    /// bounding box is fully inside, the per-pixel bounds test is skipped.
    #[inline(always)]
    pub fn for_each_pixel_index_in_segment(
        &self,
        segment: &Segment<f32>,
        mut visit: impl FnMut(usize),
    ) {
        let width = self.width as i64;
        let height = self.height as i64;

        let x0 = segment.start.x.floor() as i64;
        let y0 = segment.start.y.floor() as i64;
        let x1 = segment.end.x.floor() as i64;
        let y1 = segment.end.y.floor() as i64;

        let dx = x1 - x0;
        let dy = y1 - y0;
        let step_x = dx.signum();
        let step_y = dy.signum();

        // Major axis: the one with the largest absolute delta. Ties go to x.
        let (major_count, minor_count, major, minor) = if dx.abs() >= dy.abs() {
            (dx.abs(), dy.abs(), (step_x, 0), (0, step_y))
        } else {
            (dy.abs(), dx.abs(), (0, step_y), (step_x, 0))
        };

        let inside =
            x0.min(x1) >= 0 && y0.min(y1) >= 0 && x0.max(x1) < width && y0.max(y1) < height;

        let (mut x, mut y) = (x0, y0);
        let mut diff = minor_count - major_count;

        if inside {
            for _ in 0..major_count {
                visit((y * width + x) as usize);
                if diff >= 0 {
                    x += minor.0;
                    y += minor.1;
                    diff -= major_count;
                }
                diff += minor_count;
                x += major.0;
                y += major.1;
            }
        } else {
            for _ in 0..major_count {
                if x >= 0 && y >= 0 && x < width && y < height {
                    visit((y * width + x) as usize);
                }
                if diff >= 0 {
                    x += minor.0;
                    y += minor.1;
                    diff -= major_count;
                }
                diff += minor_count;
                x += major.0;
                y += major.1;
            }
        }
    }
}
