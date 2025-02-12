use crate::{
    color::Rgb,
    geometry::{Rect, Segment},
    image::{self, Image, PixelData},
    Float,
};
use num_traits::AsPrimitive;
use std::ops::{Deref, DerefMut};

mod state;

pub use state::State;

pub struct Map<I, L, S> {
    state: State<I, L>,
    pub(crate) weights: Vec<S>,
}

impl<I, L, S> Deref for Map<I, L, S> {
    type Target = State<I, L>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<I, L, S> DerefMut for Map<I, L, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<I, L, S> From<Map<I, L, S>> for super::Named {
    fn from(value: Map<I, L, S>) -> Self {
        value.state.into()
    }
}

impl<I, L, S: Float> Map<I, L, S> {
    pub fn new(image: &Image<S>, config: State<I, L>) -> Self {
        Self {
            state: config,
            weights: vec![S::ZERO; image.width * image.height],
        }
    }

    unsafe fn blur(&mut self, rect: &Rect, sigma: usize)
    where
        usize: AsPrimitive<S>,
    {
        let image = PixelData::from_raw(std::mem::replace(&mut self.weights, Vec::new()), *rect);
        self.weights = image::blur(&image, sigma);
    }

    pub(crate) unsafe fn compute(
        &mut self,
        image: &Image<S>,
        contrast: S,
        blur_radius: usize,
        color: &Rgb<S>,
    ) where
        usize: AsPrimitive<S>,
    {
        self.blur(image.deref(), blur_radius);
        for (idx, pixel) in image.pixels().iter().enumerate() {
            let p = self.weights.get_unchecked_mut(idx);
            *p = contrast * *p + (S::ONE - contrast) * (S::THREE - pixel.distance(&color));
        }
    }

    pub(crate) fn calculate_weight(&self, segment: &Segment<S>, rect: &Rect) -> S
    where
        usize: AsPrimitive<S>,
    {
        let mut weight = S::ZERO;
        let mut count = S::ZERO;
        for idx in rect.get_pixel_indexes_in_segment(segment) {
            let delta = unsafe { *self.weights.get_unchecked(idx) };
            weight += delta;
            count += S::ONE;
        }
        if count > S::ZERO {
            weight / count.as_()
        } else {
            -S::INFINITY
        }
    }
}

impl<I, L, S> Map<I, L, S> {
    pub fn weights(&mut self) -> &mut [S] {
        &mut self.weights
    }
}
