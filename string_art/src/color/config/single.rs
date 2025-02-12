use num_traits::AsPrimitive;

use super::{map_builder::Builder as MapBuilder, NailIndexOutOfRangeError};
use crate::{
    color::{self, mapping},
    image::{Dither, Image},
    verboser, Float, NailTable,
};

pub struct SingleColorPalette {
    color: color::Named,
    threads: usize,
}

impl<'a, S: Float> super::Config<'a, S> for SingleColorPalette
where
    u8: AsPrimitive<S>,
    usize: AsPrimitive<S>,
{
    type Handle<I: 'a, L: 'a> = SingleColorHandle<I, L, S>;

    type Error = NailIndexOutOfRangeError;

    fn into_color_handle<I: 'a + Default, L: 'a + Default>(
        self,
        image: &Image<S>,
        blur_radius: usize,
        contrast: S,
    ) -> Result<Self::Handle<I, L>, Self::Error> {
        let mut weight = MapBuilder::from(color::Map::new(image, self.color.into()));
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
    }
}

pub struct SingleColorHandle<I, L, S> {
    color: color::Map<I, L, S>,
    count: usize,
}

unsafe impl<'a, I: 'a, L: 'a, S: 'a> super::Handle<'a, I, L, S> for SingleColorHandle<I, L, S> {
    type Owner = [color::Map<I, L, S>; 1];

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

    fn colors(&self) -> &[color::Map<I, L, S>; 1] {
        core::array::from_ref(&self.color)
    }

    fn colors_mut(&mut self) -> &mut [color::Map<I, L, S>; 1] {
        core::array::from_mut(&mut self.color)
    }
}
