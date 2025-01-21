use std::ops::{Deref, DerefMut};

use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};

use crate::{
    ditherer::{DitherCounter, Ditherer}, line_config::LineItemConfig, line_selector::{self, LineSelector}, verboser::Verboser, AsLab, Float, Image, Lab
};

#[derive(Clone, Serialize, Deserialize)]
pub struct AutoLineConfig<
    S,
    G = Vec<AutoLineGroupConfig<S>>,
    C = Vec<usize>,
> {
    pub groups: G,
    pub threads: usize,
    _panthom: std::marker::PhantomData<(S, C)>,
}

impl<S, G, C> AutoLineConfig<S, G, C>{
    pub fn new(groups: G, threads: usize) -> Self {
        AutoLineConfig {
            groups,
            threads,
            _panthom: std::marker::PhantomData,
        }
    }
}

impl<S, G, C> AutoLineConfig<S, G, C>
where
    G: AsRef<[AutoLineGroupConfig<S, C>]>,
    C: AsRef<[usize]>,
{    
    pub fn bake<T: Float, R: FromIterator<impl FromIterator<I>>, I: From<LineItemConfig>>(
        &self,
        image: &Image<T>,
        palette: &[impl AsLab<T>],
        verboser: &mut impl Verboser
    ) -> Result<R, line_selector::Error>
    where
        S: AsPrimitive<T>,
        usize: AsPrimitive<T>,
    {
        let iter = palette.iter();
        let mut dither_counters: Vec<AutoLineDitherCounter<T>> = iter
            .map(|color| AutoLineDitherCounter {
                lab: color.as_lab(),
                weight: T::ZERO,
                pixel_count: T::ZERO,
            })
            .collect();
        match Ditherer::floyd_steinberg(dither_counters.as_mut_slice()).dither(&mut image.clone(), verboser) {
            Ok(_) => {
                for group in self.groups.as_ref() {
                    let weight = group.weight;
                    for &index in group.colors.as_ref() {
                        match dither_counters.get_mut(index) {
                            Some(counter) => counter.weight += weight.as_(),
                            None => return Err(line_selector::Error),
                        }
                    }
                }
                let pixel_count = image.pixels().len().as_();
                let threads = self.threads.as_();

                Ok(self
                    .groups
                    .as_ref()
                    .iter()
                    .map(|group| {
                        group
                            .colors
                            .as_ref()
                            .iter()
                            .map(|&idx| {
                                let counter = unsafe { dither_counters.get_unchecked(idx) };
                                let prop = (group.weight.as_() * counter.pixel_count)
                                    / (pixel_count * counter.weight);
                                I::from(LineItemConfig::new(
                                    idx,
                                    (threads * prop).to_usize().unwrap(),
                                ))
                            })
                            .collect()
                    })
                    .collect())
            }
            Err(_) => core::iter::empty().collect(),
        }
    }
}

unsafe impl<S, T, G, C> line_selector::Builder<T> for AutoLineConfig<S, G, C>
where
    usize: AsPrimitive<T>,
    S: AsPrimitive<T>,
    T: Float,
    G: AsRef<[AutoLineGroupConfig<S, C>]>,
    C: AsRef<[usize]>,
{
    fn build_line_selector(
        &self,
        image: &Image<T>,
        palette: &[impl AsLab<T>],
        verboser: &mut impl Verboser
    ) -> Result<LineSelector, line_selector::Error> {
        self.bake(image, palette, verboser)
    }
}

impl<S> Deref for AutoLineConfig<S> {
    type Target = Vec<AutoLineGroupConfig<S>>;

    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

impl<S> DerefMut for AutoLineConfig<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.groups
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct AutoLineGroupConfig<S, C = Vec<usize>> {
    pub colors: C,
    pub weight: S,
}

impl<S, C> AutoLineGroupConfig<S, C> {
    pub fn new(colors: C, weight: S) -> Self {
        AutoLineGroupConfig { colors, weight }
    }
}

impl<S> Deref for AutoLineGroupConfig<S> {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.colors
    }
}

impl<S> DerefMut for AutoLineGroupConfig<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.colors
    }
}

struct AutoLineDitherCounter<S> {
    lab: Lab<S>,
    weight: S,
    pixel_count: S,
}

impl<S: Float> DitherCounter<S> for AutoLineDitherCounter<S> {
    fn color(&self) -> Lab<S> {
        self.lab
    }

    fn add_pixel(&mut self) {
        self.pixel_count += S::ONE;
    }
}

// #[derive(Debug, thiserror::Error)]
// pub enum Error {
//     #[error("Invalid group index")]
//     InvalidGroupIndex,
//     #[error(transparent)]
//     Ditherer(ditherer::Error),
// }
