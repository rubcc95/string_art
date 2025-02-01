use crate::{blur, geometry::Segment, grid::Grid, AsRgb, Float, Image, PixelData, Rgb};
use num_traits::AsPrimitive;
use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct ColorConfig<L, S = u8> {
    pub name: String,
    pub color: Rgb<S>,
    pub nail: usize,
    pub link: L,
}

impl<L, S: Float> From<ColorConfig<L>> for ColorConfig<L, S>
where
    u8: AsPrimitive<S>,
{
    fn from(value: ColorConfig<L>) -> Self {
        Self {
            name: value.name,
            color: value.color.as_rgb(),
            nail: value.nail,
            link: value.link,
        }
    }
}

impl<L> ColorConfig<L> {
    pub fn new(name: String, color: Rgb, nail: usize, link: L) -> Self {
        Self {
            name,
            color,
            nail,
            link,
        }
    }
}

pub struct ColorWeight<L, S> {
    map: ColorMap<L, S>,
    rgb: Rgb<S>,
    pub(crate) count: usize,
}

impl<L, S: Float> From<ColorMap<L, S>> for ColorWeight<L, S>
where
    u8: AsPrimitive<S>,
{
    fn from(map: ColorMap<L, S>) -> Self {
        Self {
            count: 0,
            rgb: map.color.as_rgb(),
            map,
        }
    }
}

impl<L, S> Deref for ColorWeight<L, S> {
    type Target = ColorMap<L, S>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<L, S> DerefMut for ColorWeight<L, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<L, S> ColorWeight<L, S> {
    pub fn color(&self) -> &Rgb<S> {
        &self.rgb
    }
}

impl<L, S: Float> ColorWeight<L, S> {
    pub unsafe fn blur(&mut self, grid: &Grid, sigma: usize)
    where
        usize: AsPrimitive<S>,
    {
        let image = PixelData::from_raw(std::mem::replace(&mut self.weights, Vec::new()), *grid);
        self.weights = blur::linear_blur(&image, sigma);
    }

    pub unsafe fn compute(&mut self, image: &Image<S>, contrast: S, blur_radius: usize)
    where
        usize: AsPrimitive<S>,
    {
        self.blur(image.deref(), blur_radius);
        self.map.compute_with_color(image, contrast, &self.rgb);
    }

    pub unsafe fn compute_gray_scale(&mut self, image: &Image<S>, contrast: S, blur_radius: usize)
    where
        usize: AsPrimitive<S>,
    {
        self.blur(image.deref(), blur_radius);
        self.map
            .compute_with_color(image, contrast, &Rgb(S::ZERO, S::ZERO, S::ZERO));
    }
}

pub struct ColorMap<L, S> {
    config: ColorConfig<L>,
    pub(crate) weights: Vec<S>,
}

// impl<L, S: Float> AsRgb<S> for ColorMap<L, S>
// where
//     u8: AsPrimitive<S>,
// {
//     fn as_rgb(&self) -> Rgb<S> {
//         self.config.as_rgb()
//     }
// }

impl<L, S> Deref for ColorMap<L, S> {
    type Target = ColorConfig<L>;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl<L, S> DerefMut for ColorMap<L, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

impl<L, S> From<ColorWeight<L, S>> for ColorMap<L, S> {
    fn from(weights: ColorWeight<L, S>) -> Self {
        weights.map
    }
}

impl<L, S: Float> ColorMap<L, S> {
    pub fn new(image: &Image<S>, config: ColorConfig<L>) -> Self {
        Self {
            config,
            weights: vec![S::ZERO; image.width * image.height],
        }
    }

    unsafe fn compute_with_color(&mut self, image: &Image<S>, contrast: S, color: &Rgb<S>) {
        for (idx, pixel) in image.pixels().iter().enumerate() {
            let p = self.weights.get_unchecked_mut(idx);
            *p = contrast * *p + (S::ONE - contrast) * (S::THREE - pixel.distance(&color));
        }
    }

    pub(crate) fn calculate_weight(&self, segment: &Segment<S>, grid: &Grid) -> S
    where
        usize: AsPrimitive<S>,
    {
        let mut weight = S::ZERO;
        let mut count = S::ZERO;
        for idx in grid.get_pixel_indexes_in_segment(segment) {
            let delta = unsafe { *self.weights.get_unchecked(idx) };
            weight += delta;
            count += S::ONE;
        }
        if count > S::ZERO {
            weight / count as S
        } else {
            -S::INFINITY
        }
    }
}

impl<L, S> ColorMap<L, S> {
    pub fn weights(&mut self) -> &mut [S] {
        &mut self.weights
    }
}
