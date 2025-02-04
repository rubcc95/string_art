use crate::{
    color,
    image::Image,
    slice::SliceOwner,
};

pub mod single;
pub mod multi;
mod map_builder;

pub trait Config<'a, L: 'a, S: 'a> {
    type Handle: Handle<'a, L, S>;
    type Error: core::error::Error;

    fn into_color_handle(
        self,
        image: &Image<S>,
        nail_count: usize,
        blur_radius: usize,
        contrast: S,
    ) -> Result<Self::Handle, Self::Error>;
}

//SAFETY: Must ensure that select_next index is always < colors().len()
pub unsafe trait Handle<'a, L: 'a, S: 'a> {
    type Owner: SliceOwner<'a, Item = color::Map<L, S>>;

    fn select_next(&mut self) -> Option<usize>;

    fn into_colors(self) -> Self::Owner;

    fn colors(&self) -> &<Self::Owner as SliceOwner<'a>>::Slice;

    fn colors_mut(&mut self) -> &mut <Self::Owner as SliceOwner<'a>>::Slice;
}

#[derive(Debug, thiserror::Error)]
#[error("Start nail index is out of range.")]
pub struct NailIndexOutOfRangeError;
