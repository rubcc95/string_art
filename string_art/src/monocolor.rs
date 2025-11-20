use crate::{
    Pipeline, WeightMap, board::ValidBoard, color, computation::*, math::*, pipeline::PipelineLayer,
};

use derive_more::Deref;
#[cfg(feature = "image")]
use pixel_to_frac::*;

#[cfg(feature = "image")]
mod pixel_to_frac {
    use string_art_math::{Frac, Frac8, Frac16, Frac32, Frac64};

    pub trait SubpixelToFrac<F> {
        fn to_frac(self) -> F;
    }

    macro_rules! subpixel_to_frac_impl {
        ($from:ty => $to:ty) => {
            impl SubpixelToFrac<$to> for $from {
                #[inline]
                fn to_frac(self) -> $to {
                    <$to>::from_bits(self)
                }
            }
        };
        ($from:ty => $to:ty, $replicate:expr) => {
            impl SubpixelToFrac<$to> for $from {
                #[inline]
                fn to_frac(self) -> $to {
                    <$to>::from_bits(self as <$to as Frac>::Bits) * $replicate
                }
            }
        };

        ($from:ty => $to:ty, d $shift:expr) => {
            impl SubpixelToFrac<$to> for $from {
                #[inline]
                fn to_frac(self) -> $to {
                    <$to>::from_bits((self >> $shift) as _)
                }
            }
        };
    }

    subpixel_to_frac_impl!(u8 => Frac8);
    subpixel_to_frac_impl!(u8 => Frac16, 0x0101);
    subpixel_to_frac_impl!(u8 => Frac32, 0x01010101);
    subpixel_to_frac_impl!(u8 => Frac64, 0x0101);

    subpixel_to_frac_impl!(u16 => Frac8, d 8);
    subpixel_to_frac_impl!(u16 => Frac16);
    subpixel_to_frac_impl!(u16 => Frac32, 0x00010001);
    subpixel_to_frac_impl!(u16 => Frac64, 0x0001000100010001);

    subpixel_to_frac_impl!(u32 => Frac8, d 24);
    subpixel_to_frac_impl!(u32 => Frac16, d 16);
    subpixel_to_frac_impl!(u32 => Frac32);
    subpixel_to_frac_impl!(u32 => Frac64, 0x0000000100000001);

    subpixel_to_frac_impl!(u64 => Frac8, d 56);
    subpixel_to_frac_impl!(u64 => Frac16, d 48);
    subpixel_to_frac_impl!(u64 => Frac32, d 32);
    subpixel_to_frac_impl!(u64 => Frac64);
}

#[derive(Deref, Debug, Clone)]
pub struct Monocolor<S> {
    pub grid: string_art_grid::Grid<S>,
}

impl<S: Frac> Monocolor<S> {
    #[cfg(feature = "image")]
    pub fn from_image(
        image: &impl image::GenericImageView<Pixel: image::Pixel<Subpixel: SubpixelToFrac<S>>>,
    ) -> Self {
        use image::Pixel;
        use string_art_grid::Grid;

        Self {
            grid: unsafe {
                use string_art_geometry::Rect;

                Grid::from_raw(
                    image
                        .pixels()
                        .map(|(_, _, mut pixel)| {
                            pixel.invert();
                            let [r] = pixel.to_luma().0;
                            r.to_frac()
                        })
                        .collect(),
                    Rect::new(image.width() as usize, image.height() as usize),
                )
            },
        }
    }
}

impl<F> WeightMap for Monocolor<F> {
    type Weight = F;

    fn get(&self, pixel: string_art_geometry::Point<usize>) -> Option<&Self::Weight> {
        self.grid.get(pixel)
    }

    fn get_mut(&mut self, pixel: string_art_geometry::Point<usize>) -> Option<&mut Self::Weight> {
        self.grid.get_mut(pixel)
    }
}

impl<F: Frac> Pipeline for Monocolor<F> {
    type MapId = ();

    type Runtime<A> = MonocolorRuntime<A, F>;

    type Weight = F;

    type Layer<'a, A>
        = &'a mut MonocolorRuntime<A, F>
    where
        Self: 'a,
        A: 'a;

    fn init<B: ValidBoard>(
        self,
        batched: &mut BatchedBoard<B, Self::Weight>,
    ) -> Self::Runtime<B::Anchor> {
        MonocolorRuntime {
            anchor: batched
                .get_best_line(&self, B::Anchor::default())
                .map_or_else(B::Anchor::default, |step| step.anchor),
            monocolor: self,
        }
    }

    fn next<A>(runtime: &mut Self::Runtime<A>) -> Option<Self::Layer<'_, A>> {
        Some(runtime)
    }
}

pub struct MonocolorRuntime<A, F> {
    monocolor: Monocolor<F>,
    anchor: A,
}

impl<A, F> WeightMap for &mut MonocolorRuntime<A, F> {
    type Weight = F;

    fn get(&self, pixel: string_art_geometry::Point<usize>) -> Option<&Self::Weight> {
        self.monocolor.grid.get(pixel)
    }

    fn get_mut(&mut self, pixel: string_art_geometry::Point<usize>) -> Option<&mut Self::Weight> {
        self.monocolor.grid.get_mut(pixel)
    }
}

impl<A, F> PipelineLayer for &mut MonocolorRuntime<A, F> {
    type MapId = ();

    type AnchorId = A;

    fn anchor(&self) -> &Self::AnchorId {
        &self.anchor
    }

    fn set_anchor(&mut self, id: Self::AnchorId) {
        self.anchor = id;
    }

    fn id(&self) -> Self::MapId {
        ()
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
// impl<A: Clone + CondSend + CondSync, S: Fixed> Pipeline for Monocolor<A, S> {
//     type AnchorId = A;

//     type Color = Color;

//     type Index = MonocolorIndex;

//     type Fixed = S;

//     fn color_map(&self, _: &Self::Index) -> &ColorMap<A, Color, S::Frac> {
//         &self.map
//     }

//     fn color_map_mut(&mut self, _: &Self::Index) -> &mut ColorMap<A, Color, S::Frac> {
//         &mut self.map
//     }

//     fn color_maps(&self) -> &[ColorMap<A, Color, S::Frac>] {
//         core::slice::from_ref(&self.map)
//     }

//     fn color_maps_mut(&mut self) -> &mut [ColorMap<A, Color, S::Frac>] {
//         core::slice::from_mut(&mut self.map)
//     }
// }
