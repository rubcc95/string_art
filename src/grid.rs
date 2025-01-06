use num_traits::Unsigned;

use crate::geometry::Point;

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

impl<T: Unsigned + PartialOrd + Copy> Grid<T> {
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
