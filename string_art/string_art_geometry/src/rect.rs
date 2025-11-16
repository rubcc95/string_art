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
    pub fn get_pixel_indexes_in_segment<F: Float+ std::fmt::Debug  + 'static>(
        &self,
        seg: &Segment<F>,
    ) -> impl Iterator<Item = T>
    where
        usize: AsPrimitive<F>,
    {
        self.bresenham(seg)
            .filter_map(|point| self.index_of(point))
    }

    pub fn bresenham<F: Float + std::fmt::Debug + 'static>(
        &self,
        seg: &Segment<F>,
    ) -> impl Iterator<Item = Point<T>> + use<F, T> {
        let segment = seg.floor(); 
        let casted = segment.cast::<isize>();
        let casted = match casted{
            Some(casted) => casted,
            None => panic!("{segment:?} is not castable"),
        };
        let bresenham = casted.bresenham();
        bresenham.filter_map(|p| p.cast::<T>())
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
