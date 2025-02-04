use num_traits::AsPrimitive;

use crate::{
    color::{self, mapping},
    image::{self, Image},
    slice::{Slice, SliceOwner},
    verboser, Float,
};
use super::map_builder::{Builder as MapBuilder, UnsafeDitherPalette};

pub mod ordering;

pub use ordering::auto;
pub use ordering::manual;
pub use ordering::auto::Auto as Auto;
pub use ordering::manual::Manual as Manual;


pub struct Config<C, B> {
    colors: C,
    ordering: B,
}

impl<C, B> Config<C, B> {
    #[inline]
    pub fn new(colors: C, ordering: B) -> Self {
        Self { colors, ordering }
    }
}

impl<'a, C: 'a, B, I: 'a, L: 'a, S> super::Config<'a, L, S> for Config<C, B>
where
    C: SliceOwner<'a, Item = mapping::State<L>>,
    B: ordering::Builder<'a, S, Groups: SliceOwner<'a, Item = ordering::Group<I>>>,
    I: SliceOwner<'a, Item = ordering::Item>,
    S: Float,
    u8: AsPrimitive<S>,
    usize: AsPrimitive<S>,
{
    type Error = Error;
    type Handle = Handle<
        <C::Map<'a, MapBuilder<L, S>> as SliceOwner<'a>>::Map<'a, color::Map<L, S>>,
        B::Groups,
    >;

    fn into_color_handle(
        self,
        image: &Image<S>,
        nail_count: usize,
        blur_radius: usize,
        contrast: S,
    ) -> Result<Self::Handle, Self::Error> {
        let mut weights = self.colors.try_map(|color| {
            if nail_count > color.nail {
                Ok(MapBuilder::from(color::Map::new(image, color)))
            } else {
                Err(Error::NailIndexOutOfRange(
                    super::NailIndexOutOfRangeError,
                ))
            }
        })?;
        let mut ditherer = image::Dither::floyd_steinberg();
        ditherer
            .dither(
                //SAFETY: Wheights are builded with the image used for dithering. Buffers will match.
                unsafe { UnsafeDitherPalette::from_slice(weights.as_mut_slice()) },
                &mut image.clone(),
                &mut verboser::Silent,
            )
            .unwrap();
        for weight in weights.as_mut_slice().raw_mut_slice() {
            unsafe {
                weight.compute(image, contrast, blur_radius);
            }
        }
        let handle = self
            .ordering
            .build_handle(image, weights.as_slice(), &mut verboser::Silent)
            .map_err(Error::ColorOrdering)?;
        let color_maps = weights.map(color::Map::from);
        Ok(Handle {
            colors: color_maps,
            ordering: handle,
        })
    }
}

pub struct Handle<C, G> {
    colors: C,
    ordering: ordering::Config<G>,
}

unsafe impl<'a, C, G, I: 'a, L: 'a, S: 'a> super::Handle<'a, L, S> for Handle<C, G>
where
    C: SliceOwner<'a, Item = color::Map<L, S>>,
    G: SliceOwner<'a, Item = ordering::Group<I>>,
    I: SliceOwner<'a, Item = ordering::Item>,
{
    type Owner = C;

    #[inline]
    fn select_next(&mut self) -> Option<usize> {
        self.ordering.select_next()
    }

    #[inline]
    fn into_colors(self) -> Self::Owner {
        self.colors
    }

    #[inline]
    fn colors(&self) -> &C::Slice {
        self.colors.as_slice()
    }

    #[inline]
    fn colors_mut(&mut self) -> &mut C::Slice {
        self.colors.as_mut_slice()
    }
}


#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    ColorOrdering(ordering::LALAError),
    NailIndexOutOfRange(super::NailIndexOutOfRangeError),
}