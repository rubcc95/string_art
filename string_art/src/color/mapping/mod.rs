use crate::{
    color::Rgb,
    geometry::Segment,
    grid::Grid,
    image::{self, Image, PixelData},
    Float,
};
use num_traits::AsPrimitive;
use std::ops::{Deref, DerefMut};

mod state;

pub use state::State;

pub struct Map<L, S> {
    state: State<L>,
    pub(crate) weights: Vec<S>,
}

impl<L, S> Deref for Map<L, S> {
    type Target = State<L>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<L, S> DerefMut for Map<L, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<L, S> From<Map<L, S>> for super::Named {
    fn from(value: Map<L, S>) -> Self {
        value.state.into()
    }
}

impl<L, S: Float> Map<L, S> {
    pub fn new(image: &Image<S>, config: State<L>) -> Self {
        Self {
            state: config,
            weights: vec![S::ZERO; image.width * image.height],
        }
    }

    unsafe fn blur(&mut self, grid: &Grid, sigma: usize)
    where
        usize: AsPrimitive<S>,
    {
        let image = PixelData::from_raw(std::mem::replace(&mut self.weights, Vec::new()), *grid);
        self.weights = image::blur(&image, sigma);
    }

    pub(crate) unsafe fn compute(&mut self, image: &Image<S>, contrast: S, blur_radius: usize, color: &Rgb<S>)
    where
        usize: AsPrimitive<S>,
    {
        self.blur(image.deref(), blur_radius);
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
            weight / count.as_()
        } else {
            -S::INFINITY
        }
    }
}

impl<L, S> Map<L, S> {
    pub fn weights(&mut self) -> &mut [S] {
        &mut self.weights
    }
}
