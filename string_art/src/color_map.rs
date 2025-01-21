use std::ops::Deref;
use num_traits::AsPrimitive;
use palette::{color_difference::EuclideanDistance, FromColor, Srgb};
use crate::{geometry::Segment, grid::Grid, AsLab, Float, Image, Lab};


#[derive(Clone)]
pub struct ColorConfig<L> {
    pub name: String,
    pub color: (u8, u8, u8),
    pub nail: usize,
    pub link: L,
}

impl<L, S: Float> AsLab<S> for ColorConfig<L> where u8: AsPrimitive<S>{
    fn as_lab(&self) -> Lab<S> {
        self.color.as_lab()
    }
}

impl<L> ColorConfig<L> {
    pub fn new(name: String, color: (u8, u8, u8), nail: usize, link: L) -> Self {
        Self {
            name,
            color,
            nail,
            link,
        }
    }
}

#[derive(Clone)]
pub struct LabColorMapSettings<S, L> {
    lab: Lab<S>,
    inner: ColorConfig<L>,
}

impl<S: Float, L> From<ColorConfig<L>> for LabColorMapSettings<S, L>
where
    u8: AsPrimitive<S>,
{
    fn from(settings: ColorConfig<L>) -> Self {
        Self {
            lab: Lab::from_color(Srgb::new(
                settings.color.0.as_() / S::TWO_FIVE_FIVE,
                settings.color.1.as_() / S::TWO_FIVE_FIVE,
                settings.color.2.as_() / S::TWO_FIVE_FIVE,
            )),
            inner: settings,
        }
    }
}

impl<S, L> Deref for LabColorMapSettings<S, L> {
    type Target = ColorConfig<L>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ColorMap<S, L> {
    settings: ColorConfig<L>,
    weights: Vec<S>,
    pub(crate) curr_nail: usize,
    pub(crate) curr_link: L,
}


impl<S: Float, L> AsLab<S> for ColorMap<S, L> where u8: AsPrimitive<S>{
    fn as_lab(&self) -> Lab<S> {
        self.settings.as_lab()
    }
}

impl<S, L> Deref for ColorMap<S, L> {
    type Target = ColorConfig<L>;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl<S: Float, L: Copy> ColorMap<S, L> {
    pub fn new(image: &Image<S>, settings: LabColorMapSettings<S, L>) -> Self {
        Self {
            weights: image
                .pixels()
                .iter()
                .map(|pixel_color| S::SQRT140050 - pixel_color.distance(settings.lab))
                .collect(),
            curr_nail: settings.nail,
            curr_link: settings.link,
            settings: settings.inner,
        }
    }

    pub (crate) fn calculate_weight(&self, segment: &Segment<S>, grid: &Grid) -> S {
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

    pub fn weights(&mut self) -> &mut [S] {
        &mut self.weights
    }   
}
