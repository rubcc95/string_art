use std::ops::{Deref, DerefMut};
use num_traits::AsPrimitive;

use crate::{
    color::{AsRgb, Map, Rgb},
    image::{dither, Image},
    slice::Slice,
    Float,
};

pub struct Builder<I, L, S> {
    map: Map<I, L, S>,
    rgb: Rgb<S>,
    pub(crate) count: usize,
}

impl<I, L, S> From<Builder<I, L, S>> for Map<I, L, S> {
    #[inline]
    fn from(weights: Builder<I, L, S>) -> Self {
        weights.map
    }
}

impl<I, L, S: Float> From<Map<I, L, S>> for Builder<I, L, S>
where
    u8: AsPrimitive<S>,
{
    #[inline]
    fn from(map: Map<I, L, S>) -> Self {
        Self {
            count: 0,
            rgb: map.as_rgb(),
            map,
        }
    }
}

impl<I, L, S> Deref for Builder<I, L, S> {
    type Target = Map<I, L, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<I, L, S> DerefMut for Builder<I, L, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<I, L, S> Builder<I, L, S> {
    #[inline]
    pub fn color_normalized(&self) -> &Rgb<S> {
        &self.rgb
    }
}

impl<I, L, S: Float> Builder<I, L, S> {
    #[inline]
    pub unsafe fn compute(&mut self, image: &Image<S>, contrast: S, blur_radius: usize)
    where
        usize: AsPrimitive<S>,
    {
        self.map.compute(image, contrast, blur_radius, &self.rgb);
    }

    #[inline]
    pub unsafe fn compute_gray_scale(&mut self, image: &Image<S>, contrast: S, blur_radius: usize)
    where
        usize: AsPrimitive<S>,
    {
        self.map.compute(
            image,
            contrast,
            blur_radius,
            &Rgb(S::ZERO, S::ZERO, S::ZERO),
        );
    }
}

struct GrayScale<S>(S);

impl<S: Float> GrayScale<S> {
    const BLACK: Rgb<S> = Rgb(S::ZERO, S::ZERO, S::ZERO);
    const WHITE: Rgb<S> = Rgb(S::ONE, S::ONE, S::ONE);
}

impl<'a, I, L: 'a, S: Float> dither::Palette<'a, S> for Builder<I, L, S> {
    type Color<'u>
        = SingleDitherUnit<'u, I, L, S>
    where
        Self: 'u,
        'a: 'u;

    #[inline]
    fn iter<'u>(&'u mut self) -> impl Iterator<Item = Self::Color<'u>>
    where
        Self: 'u,
        'a: 'u,
    {
        [
            SingleDitherUnit {
                color: GrayScale::BLACK,
                weight: Some(self),
            },
            SingleDitherUnit {
                color: GrayScale::WHITE,
                weight: None,
            },
        ]
        .into_iter()
    }
}

pub struct SingleDitherUnit<'a, I, L, S> {
    color: Rgb<S>,
    weight: Option<&'a mut Builder<I, L, S>>,
}

impl<'a, I, L, S: Float> dither::Color<S> for SingleDitherUnit<'a, I, L, S> {
    #[inline]
    fn color(&self) -> Rgb<S> {
        self.color
    }

    fn set_pixel(&mut self, pixel_index: usize) {
        if let Some(weight) = &mut self.weight {
            weight.count += 1;
            *unsafe { weight.weights.get_unchecked_mut(pixel_index) } = S::THREE;
        }
    }
}

#[repr(transparent)]
pub struct UnsafeDitherPalette<T: ?Sized>(T);

//SAFETY: This struct has an unsafe dither::Palette implementation
//Is intended to be used ONLY with images matching the size buffer of the builder.
// Using this on a builder with smaller buffer size than the image will turn into UB.
impl<T: ?Sized> UnsafeDitherPalette<T> {
    #[inline]
    pub unsafe fn from_slice(slice: &mut T) -> &mut Self {
        &mut *(slice as *mut _ as *mut Self)
    }
}

impl<'a, T: ?Sized + Slice<'a, Item = Builder<I, L, S>>, I: 'a, L: 'a, S: Float>
    dither::Palette<'a, S> for UnsafeDitherPalette<T>
{
    type Color<'u>
        = &'u mut Builder<I, L, S>
    where
        'a: 'u,
        Self: 'u;

    #[inline]
    fn iter<'u>(&'u mut self) -> impl Iterator<Item = Self::Color<'u>>
    where
        'a: 'u,
        Self: 'u,
    {
        self.0.raw_mut_slice().into_iter()
    }
}

impl<'a, I, L, S: Float> dither::Color<S> for &mut Builder<I, L, S> {
    #[inline]
    fn color(&self) -> Rgb<S> {
        *self.color_normalized()
    }

    #[inline]
    fn set_pixel(&mut self, pixel_index: usize) {
        *unsafe { self.weights.get_unchecked_mut(pixel_index) } = S::THREE;
        self.count += 1;
    }
}
