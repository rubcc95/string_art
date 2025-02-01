use std::ops::{Deref, DerefMut};

use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};

use crate::{
    color_map::ColorWeight, line_config::Item, color_handle::{self, Handle}, verboser::Verboser, Float, Image
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
    pub fn bake<'a, T: Float, R: FromIterator<impl FromIterator<I>>, I: From<Item>, L: 'a>(
        &self,
        image: &Image<T>,
        weights: impl Iterator<Item = &'a ColorWeight<L, T>>,
        _: &mut impl Verboser
    ) -> Result<R, color_handle::Error>
    where
        S: AsPrimitive<T>,
        usize: AsPrimitive<T>,
    {
        let mut weights: Vec<AutoLineDitherCounter<'_, L, T>> = weights.map(|counter| {
            AutoLineDitherCounter {
                color: counter,
                weight: T::ZERO,
            }
        }).collect();
        for group in self.groups.as_ref() {
            for &index in group.colors.as_ref() {
                match weights.get_mut(index) {
                    Some(weight) => weight.weight += group.weight.as_(),
                    None => return Err(color_handle::Error),
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
                    let weight = unsafe { weights.get_unchecked(idx) };
                    let prop = (group.weight.as_() * weight.color.count.as_())
                        / (pixel_count * weight.weight);
                    I::from(Item::new(
                        idx,
                        (threads * prop).to_usize().unwrap(),
                    ))
                })
                .collect()
        })
        .collect())      
    }
}

unsafe impl<S, T, G, C> color_handle::Builder<T> for AutoLineConfig<S, G, C>
where
    usize: AsPrimitive<T>,
    S: AsPrimitive<T>,
    T: Float,
    G: AsRef<[AutoLineGroupConfig<S, C>]>,
    C: AsRef<[usize]>,
{
    fn build_line_selector<L>(
        &self,
        image: &Image<T>,
        weights: &[ColorWeight<L, T>],
        verboser: &mut impl Verboser
    ) -> Result<Handle, color_handle::Error> {
        self.bake(image, weights.iter(), verboser)
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

struct AutoLineDitherCounter<'a, L, S> {
    color: &'a ColorWeight<L, S>,
    weight: S,
    
}