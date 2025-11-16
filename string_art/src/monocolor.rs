use std::iter::FusedIterator;

use crate::{ColorMap, computation::*, color, math::*, sync::*};

pub struct Monocolor<A, S: Scalar = Scalar32> {
    pub map: ColorMap<A, Color, S::Frac>,
    pub threads: u32,
}

impl<A: Default, S: Scalar> Monocolor<A, S> {
    #[cfg(feature = "image")]
    pub fn from_image(
        image: &impl image::GenericImageView<Pixel: image::Pixel<Subpixel: IntoFrac<S::Frac>>>,
        threads: u32,
    ) -> Self {
        use image::Pixel;

        Self {
            map: ColorMap {
                anchor: A::default(),
                color: Color,
                weights: image
                    .pixels()
                    .map(|(_, _, mut pixel)| {
                        pixel.invert();
                        let [r] = pixel.to_luma().0;
                        r.into_frac()
                    })
                    .collect(),
            },
            threads,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Color;

impl color::Color for Color {
    type Unit = u8;

    fn r(&self) -> u8 {
        0
    }

    fn g(&self) -> u8 {
        0
    }

    fn b(&self) -> u8 {
        0
    }

    fn rgb(&self) -> [u8; 3] {
        [0, 0, 0]
    }
}

impl core::fmt::Display for Color {
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

impl<A: Clone + CondSend + CondSync, S: Scalar> Iterator for Monocolor<A, S> {
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

impl<A: Clone + CondSend + CondSync, S: Scalar> FusedIterator for Monocolor<A, S>{}

impl<A: Clone + CondSend + CondSync, S: Scalar> ExactSizeIterator for Monocolor<A, S>{
    fn len(&self) -> usize {
        self.threads as usize
    }
}

impl<A: Clone + CondSend + CondSync, S: Scalar> Pipeline for Monocolor<A, S> {
    type Anchor = A;

    type Color = Color;

    type Index = MonocolorIndex;

    type Scalar = S;

    fn color_map(&self, _: &Self::Index) -> &ColorMap<A, Color, S::Frac> {
        &self.map
    }

    fn color_map_mut(&mut self, _: &Self::Index) -> &mut ColorMap<A, Color, S::Frac> {
        &mut self.map
    }

    fn color_maps(&self) -> &[ColorMap<A, Color, S::Frac>] {
        core::slice::from_ref(&self.map)
    }

    fn color_maps_mut(&mut self) -> &mut [ColorMap<A, Color, S::Frac>] {
        core::slice::from_mut(&mut self.map)
    }

    // fn startup<T>(
    //     &mut self,
    //     mut f: impl FnMut(&mut ColorMap<A, Color, S::Frac>) -> T,
    // ) -> impl Iterator<Item = T> {
    //     core::iter::once(f(&mut self.map))
    // }

    // fn next<T>(
    //     &mut self,
    //     mut f: impl FnMut(MonocolorIndex, &mut ColorMap<A, Color, S::Frac>) -> T,
    // ) -> Option<T> {
    //     if self.threads > 0 {
    //         self.threads -= 1;
    //         Some(f(MonocolorIndex(()), &mut self.map))
    //     } else {
    //         None
    //     }
    // }
}

// impl<A: Clone + CondSend + CondSync, S: FracToScalar> Pipeline for Monocolor<A, S> {
//     type Anchor = A;

//     type Color = Color;

//     type Index = ();

//     type Frac = S;

//     fn startup(
//         &mut self,
//     ) -> impl Iterator<Item = (Self::Index, &mut ColorMap<A, Self::Color, Self::Frac>)> {
//         core::iter::once(((), &mut self.map))
//     }

//     fn next(
//             &mut self,
//         ) -> Option<(
//             Self::Index,
//             &mut ColorMap<A, Self::Color, Self::Frac>,
//         )> {
//         //if self.threads <
//     }

//     // fn for_each<E>(
//     //     &mut self,
//     //     mut f: impl FnMut(
//     //         Self::Index,
//     //         &mut ColorMap<A, Self::Color, Self::Frac>,
//     //     ) -> Result<(), E>,
//     // ) -> Result<(), E> {
//     //     for _ in 0..self.threads {
//     //         f((), &mut self.map)?;
//     //     }
//     //     Ok(())
//     // }

//     fn color_map(&self, _: &Self::Index) -> &ColorMap<A, Self::Color, Self::Frac> {
//         &self.map
//     }
// }
