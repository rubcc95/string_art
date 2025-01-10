use num_traits::{NumCast, Unsigned};

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

impl<T: NumCast + Unsigned + PartialOrd + Copy> Grid<T> {
    pub fn get_pixel_indexes_in_segment<F: Float>(&self, seg: Segment<F>) -> impl Iterator<Item = T> + '_ {
        self.get_pixel_coords_in_segment(seg).filter_map(|point| self.index_of(point))
    }

    pub fn get_pixel_coords_in_segment<F: Float>(&self, seg: Segment<F>) -> impl Iterator<Item = Point<T>> + '_ {
        seg.floor()
            .cast::<isize>()
            .unwrap()
            .points_between()
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
