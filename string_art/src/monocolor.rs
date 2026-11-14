use std::iter::FusedIterator;

use crate::{Color, computation::*, num::*, sync::*};

pub struct Monocolor<A> {
    map: ColorMap<A, Black>,
    threads: u32,
}

impl<A> Monocolor<A> {
    /// Weights of the pipeline's single color map.
    pub fn weights(&self) -> &[Frac] {
        &self.map.weights
    }
}

impl<A: Default> Monocolor<A> {
    #[inline]
    pub fn new(anchor: A, weights: Vec<Frac>, threads: u32) -> Self {
        Self {
            map: ColorMap {
                anchor,
                weights,
                color: Black,
            },
            threads,
        }
    }

    #[cfg(feature = "image")]
    pub fn from_image(
        image: &impl image::GenericImageView<Pixel: image::Pixel<Subpixel = u8>>,
        threads: u32,
    ) -> Self {
        use image::Pixel;

        Self {
            map: ColorMap {
                anchor: A::default(),
                color: Black,
                weights: image
                    .pixels()
                    .map(|(_, _, mut pixel)| {
                        pixel.invert();
                        let [r] = pixel.to_luma().0;
                        frac_from_u8(r)
                    })
                    .collect(),
            },
            threads,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Black;

impl Color for Black {
    type Unit = u8;

    #[inline]
    fn r(&self) -> u8 {
        0
    }

    #[inline]
    fn g(&self) -> u8 {
        0
    }

    #[inline]
    fn b(&self) -> u8 {
        0
    }
}

impl core::fmt::Display for Black {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct MonocolorIndex;

impl Into<usize> for MonocolorIndex {
    fn into(self) -> usize {
        0
    }
}

impl std::fmt::Display for MonocolorIndex {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<A> Iterator for Monocolor<A> {
    type Item = MonocolorIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.threads.checked_sub(1).map(|val| {
            self.threads = val;
            MonocolorIndex
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = self.threads as usize;
        (s, Some(s))
    }
}

impl<A> FusedIterator for Monocolor<A> {}

impl<A> ExactSizeIterator for Monocolor<A> {
    fn len(&self) -> usize {
        self.threads as usize
    }
}

impl<A: Clone + CondSend + CondSync> Pipeline for Monocolor<A> {
    type Anchor = A;

    type Color = Black;

    type Index = MonocolorIndex;

    fn color_map(&self, _: &Self::Index) -> &ColorMap<A, Black> {
        &self.map
    }

    fn color_map_mut(&mut self, _: &Self::Index) -> &mut ColorMap<A, Black> {
        &mut self.map
    }

    fn color_maps(&self) -> &[ColorMap<A, Black>] {
        core::slice::from_ref(&self.map)
    }

    fn color_maps_mut(&mut self) -> &mut [ColorMap<A, Black>] {
        core::slice::from_mut(&mut self.map)
    }
}
