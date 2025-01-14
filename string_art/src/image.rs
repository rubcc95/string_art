use crate::{geometry::Point, Float, Grid, Lab};
use image::{DynamicImage, GenericImageView, Rgb32FImage, RgbImage, Rgba32FImage, RgbaImage};
use num_traits::AsPrimitive;
use palette::{FromColor, Srgb};
use std::ops::Deref;



#[derive(Clone)]
pub struct Image<T> {
    pixels: Vec<Lab<T>>,
    grid: Grid,
}

impl<T: Float> From<DynamicImage> for Image<T>
where
    u8: AsPrimitive<T>,
{
    fn from(value: DynamicImage) -> Self {
        Self {
            pixels: value
                .pixels()
                .map(|(_, _, pixel)| {
                    Lab::from_color(Srgb::new(
                        pixel.0[0].as_() / T::TWO_FIVE_FIVE,
                        pixel.0[1].as_() / T::TWO_FIVE_FIVE,
                        pixel.0[2].as_() / T::TWO_FIVE_FIVE,
                    ))
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
                .map(|pixel| {
                    Lab::from_color(Srgb::new(
                        pixel.0[0].as_(),
                        pixel.0[1].as_(),
                        pixel.0[2].as_(),
                    ))
                })
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
                .map(|pixel| {
                    Lab::from_color(Srgb::new(
                        pixel.0[0].as_(),
                        pixel.0[1].as_(),
                        pixel.0[2].as_(),
                    ))
                })
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
                .map(|pixel| {
                    Lab::from_color(Srgb::new(
                        pixel.0[0].as_(),
                        pixel.0[1].as_(),
                        pixel.0[2].as_(),
                    ))
                })
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
                .map(|pixel| {
                    Lab::from_color(Srgb::new(
                        pixel.0[0].as_(),
                        pixel.0[1].as_(),
                        pixel.0[2].as_(),
                    ))
                })
                .collect(),
            grid: Grid {
                height: value.height() as usize,
                width: value.width() as usize,
            },
        }
    }
}

impl<T> Image<T> {
    pub fn pixels(&self) -> &[Lab<T>] {
        &self.pixels
    }

    pub fn get(&self, point: Point<usize>) -> Option<&Lab<T>> {
        self.index_of(point)
            .map(|index| unsafe { self.pixels.get_unchecked(index) })
    }

    pub fn get_mut(&mut self, point: Point<usize>) -> Option<&mut Lab<T>> {
        self.index_of(point)
            .map(|index| unsafe { self.pixels.get_unchecked_mut(index) })
    }

    pub unsafe fn get_unchecked(&self, point: Point<usize>) -> &Lab<T> {
        self.pixels.get_unchecked(self.index_of_unchecked(point))
    }

    pub unsafe fn get_unchecked_mut(&mut self, point: Point<usize>) -> &mut Lab<T> {
        self.pixels
            .get_unchecked_mut(self.grid.index_of_unchecked(point))
    }
}

impl<T> Deref for Image<T> {
    type Target = Grid;

    fn deref(&self) -> &Self::Target {
        &self.grid
    }
}
