use crate::{color::Rgb, geometry::Point, Float, Grid};
use image::{DynamicImage, GenericImageView, Rgb32FImage, RgbImage, Rgba32FImage, RgbaImage};
use num_traits::AsPrimitive;
use std::ops::Deref;

mod blur;
pub mod dither;

pub use dither::Dither;
pub use blur::blur;

#[derive(Clone)]
pub struct PixelData<T> {
    pixels: Vec<T>,
    grid: Grid,
}

impl<T> PixelData<T> {
    pub unsafe fn from_raw(pixels: Vec<T>, grid: Grid) -> Self {
        Self { pixels, grid }
    }

    pub fn new(mut builder: impl FnMut(Point<usize>) -> T, grid: Grid) -> Self {
        let mut pixels = Vec::with_capacity(grid.width * grid.height);
        unsafe { pixels.set_len(pixels.capacity()) };
        let ptr: *mut T = pixels.as_mut_ptr();
        for x in 0..grid.width {
            for y in 0..grid.height {
                let p = Point { x, y };
                unsafe { core::ptr::write(ptr.add(grid.index_of_unchecked(p)), builder(p)) };
            }
        }
        Self { pixels, grid }
    }
    pub fn pixels(&self) -> &[T] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [T] {
        &mut self.pixels
    }

    pub fn get(&self, index: impl ImageIndexer) -> Option<&T> {
        index.get(self)
    }

    pub fn get_mut(&mut self, index: impl ImageIndexer) -> Option<&mut T> {
        index.get_mut(self)
    }

    pub unsafe fn get_unchecked(&self, index: impl ImageIndexer) -> &T {
        index.get_unchecked(self)
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: impl ImageIndexer) -> &mut T {
        index.get_unchecked_mut(self)
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }
}

impl<T> Deref for PixelData<T> {
    type Target = Grid;

    fn deref(&self) -> &Self::Target {
        &self.grid
    }
}

pub type Image<T> = PixelData<Rgb<T>>;

impl<T: Float> From<DynamicImage> for Image<T>
where
    u8: AsPrimitive<T>,
{
    fn from(value: DynamicImage) -> Self {
        Self {
            pixels: value
                .pixels()
                .map(|(_, _, pixel)| {
                    Rgb(
                        pixel.0[0].as_() / T::TWO_FIVE_FIVE,
                        pixel.0[1].as_() / T::TWO_FIVE_FIVE,
                        pixel.0[2].as_() / T::TWO_FIVE_FIVE,
                    )
                })
                .collect(),
            grid: Grid {
                height: value.height() as usize,
                width: value.width() as usize,
            },
        }
    }
}

impl<T: Float> From<RgbImage> for Image<T>
where
    u8: AsPrimitive<T>,
{
    fn from(value: RgbImage) -> Self {
        Self {
            pixels: value
                .pixels()
                .map(|pixel| Rgb(pixel.0[0].as_(), pixel.0[1].as_(), pixel.0[2].as_()))
                .collect(),
            grid: Grid {
                height: value.height() as usize,
                width: value.width() as usize,
            },
        }
    }
}

impl<T: Float> From<RgbaImage> for Image<T>
where
    u8: AsPrimitive<T>,
{
    fn from(value: RgbaImage) -> Self {
        Self {
            pixels: value
                .pixels()
                .map(|pixel| Rgb(pixel.0[0].as_(), pixel.0[1].as_(), pixel.0[2].as_()))
                .collect(),
            grid: Grid {
                height: value.height() as usize,
                width: value.width() as usize,
            },
        }
    }
}

impl<T: Float> From<Rgb32FImage> for Image<T>
where
    f32: AsPrimitive<T>,
{
    fn from(value: Rgb32FImage) -> Self {
        Self {
            pixels: value
                .pixels()
                .map(|pixel| Rgb(pixel.0[0].as_(), pixel.0[1].as_(), pixel.0[2].as_()))
                .collect(),
            grid: Grid {
                height: value.height() as usize,
                width: value.width() as usize,
            },
        }
    }
}

impl<T: Float> From<Rgba32FImage> for Image<T>
where
    f32: AsPrimitive<T>,
{
    fn from(value: Rgba32FImage) -> Self {
        Self {
            pixels: value
                .pixels()
                .map(|pixel| Rgb(pixel.0[0].as_(), pixel.0[1].as_(), pixel.0[2].as_()))
                .collect(),
            grid: Grid {
                height: value.height() as usize,
                width: value.width() as usize,
            },
        }
    }
}
pub trait ImageIndexer {
    fn get_mut<T>(self, image: &mut PixelData<T>) -> Option<&mut T>;

    fn get<T>(self, image: &PixelData<T>) -> Option<&T>;

    unsafe fn get_unchecked_mut<T>(self, image: &mut PixelData<T>) -> &mut T;

    unsafe fn get_unchecked<T>(self, image: &PixelData<T>) -> &T;
}

impl ImageIndexer for usize {
    fn get_mut<T>(self, image: &mut PixelData<T>) -> Option<&mut T> {
        image.pixels.get_mut(self)
    }

    fn get<T>(self, image: &PixelData<T>) -> Option<&T> {
        image.pixels.get(self)
    }

    unsafe fn get_unchecked_mut<T>(self, image: &mut PixelData<T>) -> &mut T {
        image.pixels.get_unchecked_mut(self)
    }

    unsafe fn get_unchecked<T>(self, image: &PixelData<T>) -> &T {
        image.pixels.get_unchecked(self)
    }
}

impl ImageIndexer for Point<usize> {
    fn get_mut<T>(self, image: &mut PixelData<T>) -> Option<&mut T> {
        image
            .index_of(self)
            .map(|index| unsafe { image.pixels.get_unchecked_mut(index) })
    }

    fn get<T>(self, image: &PixelData<T>) -> Option<&T> {
        image
            .index_of(self)
            .map(|index| unsafe { image.pixels.get_unchecked(index) })
    }

    unsafe fn get_unchecked_mut<T>(self, image: &mut PixelData<T>) -> &mut T {
        let index = image.index_of_unchecked(self);
        image.pixels.get_unchecked_mut(index)
    }

    unsafe fn get_unchecked<T>(self, image: &PixelData<T>) -> &T {
        image.pixels.get_unchecked(image.index_of_unchecked(self))
    }
}
