use crate::{color, image::Image, slice::SliceOwner, NailTable};

mod map_builder;
pub mod multi;
pub mod single;

pub trait Config<'a, S: 'a> {
    type Handle<I: 'a, L: 'a>: Handle<'a, I, L, S>;
    type Error: core::error::Error;

    fn into_color_handle<I: 'a + Default, L: 'a + Default>(
        self,
        image: &Image<S>,
        blur_radius: usize,
        contrast: S,
    ) -> Result<Self::Handle<I, L>, Self::Error>;
}

//SAFETY: Must ensure that select_next index is always < colors().len()
pub unsafe trait Handle<'a, I: 'a, L: 'a, S: 'a> {
    type Owner: SliceOwner<'a, Item = color::Map<I, L, S>>;

    fn select_next(&mut self) -> Option<usize>;

    fn into_colors(self) -> Self::Owner;

    fn colors(&self) -> &<Self::Owner as SliceOwner<'a>>::Slice;

    fn colors_mut(&mut self) -> &mut <Self::Owner as SliceOwner<'a>>::Slice;
}

#[derive(Debug, thiserror::Error)]
#[error("Start nail index is out of range.")]
pub struct NailIndexOutOfRangeError;
