use num_traits::{AsPrimitive, NumCast, Unsigned};

use crate::{
    geometry::{Point, Segment},
    Float,
};

#[derive(Copy, Clone)]
pub struct Rect<T = usize> {
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

impl<S: num_traits::NumCast> Rect<S> {
    pub fn cast<I: num_traits::NumCast>(self) -> Option<Rect<I>> {
        num_traits::cast(self.width)
            .and_then(|width| num_traits::cast(self.height).map(|height| Rect { width, height }))
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

impl<T: NumCast + Unsigned + PartialOrd + Copy> Rect<T> {
    pub fn get_pixel_indexes_in_segment<F: Float>(
        &self,
        seg: &Segment<F>,
    ) -> impl Iterator<Item = T> + '_
    where
        usize: AsPrimitive<F>,
    {
        self.get_pixel_coords_in_segment(seg)
            .filter_map(|point| self.index_of(point))
    }

    pub fn get_pixel_coords_in_segment<F: Float>(
        &self,
        seg: &Segment<F>,
    ) -> impl Iterator<Item = Point<T>> + '_
    where
        usize: AsPrimitive<F>,
    {
        seg.floor()
            .cast::<isize>()
            //.linear_interpolation()
            .unwrap()
            .bresenham()
            .filter_map(|point| point.cast::<T>())
    }

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
