use num_traits::{AsPrimitive, NumCast, Unsigned};

use crate::{
    geometry::{Point, Segment},
    Float,
};

#[derive(Copy, Clone)]
pub struct Grid<T = usize> {
    pub height: T,
    pub width: T,
}

impl<T> Grid<T> {
    pub fn new(height: T, width: T) -> Self {
        Self { height, width }
    }
}

impl<T> From<Point<T>> for Grid<T> {
    fn from(point: Point<T>) -> Self {
        Self {
            height: point.y,
            width: point.x,
        }
    }
}

impl<T> From<Grid<T>> for Point<T>{
    fn from(value: Grid<T>) -> Self {
        Self {
            x: value.width,
            y: value.height,
        }
    }
}

impl<T: NumCast + Unsigned + PartialOrd + Copy> Grid<T>  {
    pub fn get_pixel_indexes_in_segment<F: Float>(&self, seg: &Segment<F>) -> impl Iterator<Item = T> + '_  where usize: AsPrimitive<F> {
        self.get_pixel_coords_in_segment(seg).filter_map(|point| self.index_of(point))
    }

    pub fn get_pixel_coords_in_segment<F: Float>(&self, seg: &Segment<F>) -> impl Iterator<Item = Point<T>> + '_ where usize: AsPrimitive<F>   {
        seg
            .floor()
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
