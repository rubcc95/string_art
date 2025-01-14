use std::ops::Deref;
use num_traits::AsPrimitive;
use palette::{color_difference::EuclideanDistance, FromColor, Srgb};
use crate::{geometry::Segment, grid::Grid, Float, Image, Lab};


#[derive(Clone)]
pub struct ColorMapSettings<L> {
    name: String,
    color: (u8, u8, u8),
    nail: usize,
    link: L,
}

impl<L> ColorMapSettings<L> {
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
    inner: ColorMapSettings<L>,
}

impl<S: Float, L> From<ColorMapSettings<L>> for LabColorMapSettings<S, L>
where
    u8: AsPrimitive<S>,
{
    fn from(settings: ColorMapSettings<L>) -> Self {
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
    type Target = ColorMapSettings<L>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ColorMap<S, L> {
    settings: ColorMapSettings<L>,
    data: Vec<S>,
    curr_nail: usize,
    curr_link: L,
}

impl<S, L> Deref for ColorMap<S, L> {
    type Target = ColorMapSettings<L>;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl<S: Float, L: Copy> ColorMap<S, L> {
    pub fn new(image: &Image<S>, settings: LabColorMapSettings<S, L>) -> Self {
        Self {
            data: image
                .pixels()
                .iter()
                .map(|pixel_color| S::SQRT140050 - pixel_color.distance(settings.lab))
                .collect(),
            curr_nail: settings.nail,
            curr_link: settings.link,
            settings: settings.inner,
        }
    }

    pub fn calculate_weight(&self, segment: Segment<S>, grid: Grid) -> S {
        let mut weight = S::ZERO;
        let mut count = S::ZERO;
        for idx in grid.get_pixel_indexes_in_segment(segment) {
            let delta = unsafe { *self.data.get_unchecked(idx) };
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
