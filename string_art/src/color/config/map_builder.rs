use std::ops::{Deref, DerefMut};

use num_traits::{AsPrimitive};

use crate::{
    color::{AsRgb, Map, Rgb},
    image::{dither, Image},
    slice::Slice,
    Float,
};

pub struct Builder<L, S> {
    map: Map<L, S>,
    rgb: Rgb<S>,
    pub(crate) count: usize,
}

impl<L, S> From<Builder<L, S>> for Map<L, S> {
    #[inline]
    fn from(weights: Builder<L, S>) -> Self {
        weights.map
    }
}

impl<L, S: Float> From<Map<L, S>> for Builder<L, S>
where
    u8: AsPrimitive<S>,
{
    #[inline]
    fn from(map: Map<L, S>) -> Self {
        Self {
            count: 0,
            rgb: map.as_rgb(),
            map,
        }
    }
}

impl<L, S> Deref for Builder<L, S> {
    type Target = Map<L, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<L, S> DerefMut for Builder<L, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<L, S> Builder<L, S> {
    #[inline]
    pub fn color_normalized(&self) -> &Rgb<S> {
        &self.rgb
    }
}

impl<L, S: Float> Builder<L, S> {
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

impl<'a, L: 'a, S: Float> dither::Palette<'a, S> for Builder<L, S> {
    type Color<'u>
        = SingleDitherUnit<'u, L, S>
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

pub struct SingleDitherUnit<'a, L, S> {
    color: Rgb<S>,
    weight: Option<&'a mut Builder<L, S>>,
}

impl<'a, L, S: Float> dither::Color<S> for SingleDitherUnit<'a, L, S> {
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

impl<'a, T: ?Sized + Slice<'a, Item = Builder<L, S>>, L: 'a, S: Float> dither::Palette<'a, S>
    for UnsafeDitherPalette<T>
{
    type Color<'u>
        = &'u mut Builder<L, S>
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

impl<'a, L, S: Float> dither::Color<S> for &mut Builder<L, S> {
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
