use std::ops::Deref;
use string_art_geometry::{Point, Rect};

#[derive(Debug)]
pub struct Grid<T> {
    rect: Rect<usize>,
    ptr: *mut T,
    cap: usize,
}

impl<T: Clone> Clone for Grid<T> {
    fn clone(&self) -> Self {
        unsafe {
            let vec = Vec::from_raw_parts(self.ptr, self.area(), self.cap);
            let cloned = vec.clone();
            core::mem::forget(vec);
            Self::from_raw(cloned, self.rect)
        }
    }
}

impl<T> Deref for Grid<T> {
    type Target = Rect<usize>;

    fn deref(&self) -> &Self::Target {
        &self.rect
    }
}

impl<T> Grid<T> {
    pub unsafe fn from_raw(mut pixels: Vec<T>, rect: Rect<usize>) -> Self {
        let ptr = pixels.as_mut_ptr();
        let cap = pixels.capacity();
        core::mem::forget(pixels); // Prevent Vec from dropping the data
        Self { rect, ptr, cap }
    }

    pub fn new(mut builder: impl FnMut(Point<usize>) -> T, rect: Rect<usize>) -> Self {
        let cap = rect.width * rect.height;
        let mut pixels: Vec<T> = Vec::with_capacity(cap);
        unsafe {
            for x in 0..rect.width {
                for y in 0..rect.height {
                    let p = Point { x, y };
                    let data = builder(p);
                    let len = pixels.len();
                    pixels.as_mut_ptr().add(len).write(data);
                    pixels.set_len(len + 1);
                }
            }
            Self::from_raw(pixels, rect)
        }
    }

    pub fn area(&self) -> usize {
        self.width * self.height
    }

    pub fn buffer(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.area()) }
    }

    pub fn buffer_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.area()) }
    }

    pub fn get(&self, index: impl Index) -> Option<&T> {
        index.get(self)
    }

    pub fn get_mut(&mut self, index: impl Index) -> Option<&mut T> {
        index.get_mut(self)
    }

    pub unsafe fn get_unchecked(&self, index: impl Index) -> &T {
        unsafe { index.get_unchecked(self) }
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: impl Index) -> &mut T {
        unsafe { index.get_unchecked_mut(self) }
    }

    pub fn rect(&self) -> &Rect<usize> {
        &self.rect
    }
}

unsafe impl<T: Send> Send for Grid<T> {}

unsafe impl<T: Sync> Sync for Grid<T> {}

impl<T> Drop for Grid<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Vec::from_raw_parts(self.ptr, self.area(), self.cap));
        }
    }
}

pub trait Index {
    fn get<T>(self, grid: &Grid<T>) -> Option<&T>;

    fn get_mut<T>(self, grid: &mut Grid<T>) -> Option<&mut T>;

    unsafe fn get_unchecked<T>(self, grid: &Grid<T>) -> &T;

    unsafe fn get_unchecked_mut<T>(self, grid: &mut Grid<T>) -> &mut T;
}

impl Index for usize {
    fn get<T>(self, grid: &Grid<T>) -> Option<&T> {
        grid.buffer().get(self)
    }

    fn get_mut<T>(self, grid: &mut Grid<T>) -> Option<&mut T> {
        grid.buffer_mut().get_mut(self)
    }

    unsafe fn get_unchecked<T>(self, grid: &Grid<T>) -> &T {
        unsafe { &*grid.ptr.add(self) }
    }

    unsafe fn get_unchecked_mut<T>(self, grid: &mut Grid<T>) -> &mut T {
        unsafe { &mut *grid.ptr.add(self) }
    }
}

impl Index for Point<usize> {
    fn get<T>(self, grid: &Grid<T>) -> Option<&T> {
        grid.index_of(self)
            .map(|index| unsafe { index.get_unchecked(grid) })
    }

    fn get_mut<T>(self, grid: &mut Grid<T>) -> Option<&mut T> {
        grid.index_of(self)
            .map(|index| unsafe { index.get_unchecked_mut(grid) })
    }

    unsafe fn get_unchecked<T>(self, grid: &Grid<T>) -> &T {
        unsafe { grid.index_of_unchecked(self).get_unchecked(grid) }
    }

    unsafe fn get_unchecked_mut<T>(self, grid: &mut Grid<T>) -> &mut T {
        unsafe { grid.index_of_unchecked(self).get_unchecked_mut(grid) }
    }
}
