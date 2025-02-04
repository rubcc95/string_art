use num_traits::AsPrimitive;

use super::{map_builder::Builder as MapBuilder, NailIndexOutOfRangeError};
use crate::{
    color::{self, mapping},
    image::{Dither, Image},
    verboser, Float,
};

pub struct SingleColorPalette<L> {
    color: mapping::State<L>,
    threads: usize,
}

impl<'a, L: 'a, S: Float> super::Config<'a, L, S> for SingleColorPalette<L>
where
    u8: AsPrimitive<S>,
    usize: AsPrimitive<S>,
{
    type Handle = SingleColorHandle<L, S>;

    type Error = NailIndexOutOfRangeError;

    fn into_color_handle(
        self,
        image: &Image<S>,
        nail_count: usize,
        blur_radius: usize,
        contrast: S,
    ) -> Result<Self::Handle, Self::Error> {
        if nail_count > self.color.nail {
            let mut weight = MapBuilder::from(color::Map::new(image, self.color));
            Dither::floyd_steinberg()
                .dither(&mut weight, &mut image.clone(), &mut verboser::Silent)
                .unwrap();
            unsafe {
                weight.compute_gray_scale(image, contrast, blur_radius);
            }

            Ok(SingleColorHandle {
                color: color::Map::from(weight),
                count: self.threads,
            })
        } else {
            Err(super::NailIndexOutOfRangeError)
        }
    }
}

pub struct SingleColorHandle<L, S> {
    color: color::Map<L, S>,
    count: usize,
}

unsafe impl<'a, L: 'a, S: 'a> super::Handle<'a, L, S> for SingleColorHandle<L, S> {
    type Owner = [color::Map<L, S>; 1];

    fn select_next(&mut self) -> Option<usize> {
        if self.count > 0 {
            self.count = unsafe { self.count.unchecked_sub(1) };
            Some(0)
        } else {
            None
        }
    }

    fn into_colors(self) -> Self::Owner {
        [self.color]
    }

    fn colors(&self) -> &[color::Map<L, S>; 1] {
        core::array::from_ref(&self.color)
    }

    fn colors_mut(&mut self) -> &mut [color::Map<L, S>; 1] {
        core::array::from_mut(&mut self.color)
    }
}
