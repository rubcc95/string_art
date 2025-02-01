//mod algorithm;
mod ditherer;
mod float;
mod image;

pub mod grid;
pub mod nails;

pub use float::Float;

pub mod geometry {
    pub mod circle;
    pub mod point;
    pub mod segment;

    pub use circle::Circle;
    pub use point::Point;
    pub use segment::Segment;
}
mod algorithm;
use grid::Grid;
mod blur;
pub mod color;
mod color_map;
pub mod multi_color_handle;
mod nail_distancer;
mod nail_table;

pub mod auto_line_config;
pub mod darkness;
pub mod line_config;

pub use algorithm::*;
pub use auto_line_config::AutoLineConfig;
pub use color::*;
pub use color_map::ColorConfig;
pub use color_map::ColorWeight;
pub use image::*;
pub use line_config::Config;
pub use nail_table::NailTable;
pub mod verboser;

mod color_handle { 
    use num_traits::AsPrimitive;

    use crate::{
        color_map::ColorMap,
        ditherer::Ditherer,
        multi_color_handle,
        nails,
        slice_owner::SliceOwner,
        verboser, ColorConfig, ColorWeight, Float, Image, NailTable,
    };

    pub trait Palette<L, S, Owner: SliceOwner> {
        type Handle: ColorHandle<L, S, Owner::Slice>;
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
    pub unsafe trait ColorHandle<L, S, Slice: ?Sized> {
        fn select_next(&mut self) -> Option<usize>;

        fn colors(&self) -> &Slice;

        fn colors_mut(&mut self) -> &mut Slice;
    }

    pub struct SingleColorPalette<L> {
        color: ColorConfig<L>,
        threads: usize,
    }

    impl<L, S: Float> Palette<L, S, [ColorMap<L, S>; 1]> for SingleColorPalette<L>
    where
        u8: AsPrimitive<S>,
        usize: AsPrimitive<S>,
    {
        type Handle = SingleColorHandle<L, S>;

        type Error = SingleColorErr;

        fn into_color_handle(
            self,
            image: &Image<S>,
            nail_count: usize,
            blur_radius: usize,
            contrast: S,
        ) -> Result<Self::Handle, Self::Error> {
            if nail_count > self.color.nail {
                let mut weight = ColorWeight::from(ColorMap::new(image, self.color));
                Ditherer::floyd_steinberg(&mut weight)
                    .dither(&mut image.clone(), &mut verboser::Silent)
                    .unwrap();
                unsafe {
                    weight.compute_gray_scale(image, contrast, blur_radius);
                }

                Ok(SingleColorHandle {
                    color: ColorMap::from(weight),
                    count: self.threads,
                })
            } else {
                Err(SingleColorErr)
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Start nail index is out of range.")]
    pub struct SingleColorErr;

    pub struct SingleColorHandle<L, S> {
        color: ColorMap<L, S>,
        count: usize,
    }

    unsafe impl<L, S> ColorHandle<L, S, [ColorMap<L, S>; 1]> for SingleColorHandle<L, S> {
        fn select_next(&mut self) -> Option<usize> {
            if self.count > 0 {
                self.count = unsafe { self.count.unchecked_sub(1) };
                Some(0)
            } else {
                None
            }
        }

        fn colors(&self) -> &[ColorMap<L, S>; 1] {
            core::array::from_ref(&self.color)
        }

        fn colors_mut(&mut self) -> &mut [ColorMap<L, S>; 1] {
            core::array::from_mut(&mut self.color)
        }
    }

    pub struct MultiColorPalette<C, B> {
        colors: C,
        builder: B,
    }

    impl<L, S, C, B> Palette<L, S, <C::Map<ColorWeight<L, S>> as SliceOwner>::Map<ColorMap<L, S>>>
        for MultiColorPalette<C, B>
    where
        u8: AsPrimitive<S>,
        usize: AsPrimitive<S>,
        S: Float,
        C: SliceOwner<Item = ColorConfig<L>>,
        B: multi_color_handle::Builder<S>,
    {
        type Handle =
            MultiColorHandle<<C::Map<ColorWeight<L, S>> as SliceOwner>::Map<ColorMap<L, S>>>;

        type Error = SingleColorErr;

        fn into_color_handle(
            self,
            image: &Image<S>,
            nail_count: usize,
            blur_radius: usize,
            contrast: S,
        ) -> Result<Self::Handle, Self::Error> {
            let mut weights = self.colors.try_map(|color| {
                if nail_count > color.nail {
                    Ok(ColorWeight::from(ColorMap::new(image, color)))
                } else {
                    Err(SingleColorErr)
                }
            })?;
            Ditherer::floyd_steinberg(weights.as_mut_slice())
                .dither(&mut image.clone().as_slice(), &mut verboser::Silent)
                .unwrap();
            for weight in weights.as_mut_slice() {
                unsafe {
                    weight.compute(image, contrast, blur_radius);
                }
            }
            let handle = self
                .builder
                .build_line_selector(image, weights.as_slice(), &mut verboser::Silent)
                .unwrap();
            let color_maps: <C::Map<ColorWeight<_, S>> as SliceOwner>::Map<ColorMap<_, _>> =
                weights.map(ColorMap::from);
            Ok(MultiColorHandle {
                colors: color_maps,
                handle,
            })
        }
    }

    pub struct MultiColorHandle<C, G> {
        colors: C,
        handle: multi_color_handle::Handle<G>,
    }

    unsafe impl<C: SliceOwner<Item = ColorMap<L, S>>, G, L, S> ColorHandle<L, S, C::Slice>
        for MultiColorHandle<C, G>
    {
        fn select_next(&mut self) -> Option<usize> {
            self.handle.select_next()
        }

        fn colors(&self) -> &C::Slice {
            self.colors.as_slice()
        }

        fn colors_mut(&mut self) -> &mut C::Slice {
            self.colors.as_mut_slice()
        }
    }
}

mod slice_owner {
    use core::slice;
    use std::{
        mem::{self, MaybeUninit},
        ptr,
    };

    pub trait Slice {
        type Item;
        type Map<S>: SliceOwner<Item = S>;

        fn len(&self) -> usize;

        fn get(&self, index: usize) -> Option<&Self::Item> {
            if self.len() < index {
                Some(unsafe { self.get_unchecked(index) })
            } else {
                None
            }
        }

        fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
            if self.len() < index {
                Some(unsafe { self.get_unchecked_mut(index) })
            } else {
                None
            }
        }

        unsafe fn get_unchecked(&self, index: usize) -> &Self::Item;

        unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Item;

        fn map<S>(&self, f: impl Fn(&Self::Item) -> S) -> Self::Map<S>;

        fn map_mut<S>(&mut self, f: impl Fn(&mut Self::Item) -> S) -> Self::Map<S>;

        fn raw_slice(&self) -> &[Self::Item];

        fn raw_mut_slice(&mut self) -> &mut [Self::Item];
    }

    impl<T, const N: usize> Slice for [T; N] {
        type Item = T;
        type Map<S> = [S; N];

        fn len(&self) -> usize {
            N
        }

        unsafe fn get_unchecked(&self, index: usize) -> &Self::Item {
            self.as_slice().get_unchecked(index)
        }

        unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Item {
            self.as_mut_slice().get_unchecked_mut(index)
        }

        fn map<S>(&self, f: impl Fn(&Self::Item) -> S) -> Self::Map<S> {
            let mut result = DropGuard::new();
            for item in self.into_iter() {
                unsafe { result.push_unchecked(f(item)) }
            }
            unsafe { result.assume_init() }
        }

        fn map_mut<S>(&mut self, f: impl Fn(&mut Self::Item) -> S) -> Self::Map<S> {
            let mut result = DropGuard::new();
            for item in self.into_iter() {
                unsafe { result.push_unchecked(f(item)) }
            }
            unsafe { result.assume_init() }
        }

        fn raw_slice(&self) -> &[Self::Item] {
            self
        }

        fn raw_mut_slice(&mut self) -> &mut [Self::Item] {
            self
        }
    }

    impl<T> Slice for [T] {
        type Item = T;
        type Map<S> = Vec<S>;

        fn len(&self) -> usize {
            self.len()
        }

        unsafe fn get_unchecked(&self, index: usize) -> &Self::Item {
            self.get_unchecked(index)
        }

        unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Self::Item {
            self.get_unchecked_mut(index)
        }

        fn map<S>(&self, f: impl Fn(&Self::Item) -> S) -> Self::Map<S> {
            self.iter().map(f).collect()
        }

        fn map_mut<S>(&mut self, f: impl Fn(&mut Self::Item) -> S) -> Self::Map<S> {
            self.iter_mut().map(f).collect()
        }

        fn raw_slice(&self) -> &[Self::Item] {
            self
        }

        fn raw_mut_slice(&mut self) -> &mut [Self::Item] {
            self
        }
    }

    pub trait SliceOwner: IntoIterator {
        type Map<S>: SliceOwner<Item = S>;
        type Slice: ?Sized + Slice<Item = Self::Item>;

        fn len(&self) -> usize;

        fn as_slice(&self) -> &Self::Slice;

        fn as_mut_slice(&mut self) -> &mut Self::Slice;

        fn map<S>(self, f: impl Fn(Self::Item) -> S) -> Self::Map<S>;

        fn try_map<S, E>(self, f: impl Fn(Self::Item) -> Result<S, E>) -> Result<Self::Map<S>, E>;
    }

    impl<T, const N: usize> SliceOwner for [T; N] {
        type Map<S> = [S; N];
        type Slice = Self;

        fn len(&self) -> usize {
            N
        }

        fn as_slice(&self) -> &Self {
            self
        }

        fn as_mut_slice(&mut self) -> &mut Self {
            self
        }

        fn map<S>(self, f: impl Fn(T) -> S) -> Self::Map<S> {
            self.map(f)
        }

        fn try_map<S, E>(self, f: impl Fn(Self::Item) -> Result<S, E>) -> Result<Self::Map<S>, E> {
            let mut result = DropGuard::new();
            for item in self.into_iter() {
                match f(item) {
                    Ok(val) => unsafe { result.push_unchecked(val) },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Ok(unsafe { result.assume_init() })
        }
    }
    
    impl<T> SliceOwner for Vec<T> {
        type Map<S> = Vec<S>;
        type Slice = [T];

        fn len(&self) -> usize {
            self.len()
        }

        fn as_slice(&self) -> &[T] {
            self
        }

        fn as_mut_slice(&mut self) -> &mut [T] {
            self
        }

        fn map<S>(self, f: impl Fn(T) -> S) -> Self::Map<S> {
            self.into_iter().map(f).collect()
        }

        fn try_map<S, E>(self, f: impl Fn(Self::Item) -> Result<S, E>) -> Result<Self::Map<S>, E> {
            self.into_iter().map(f).collect()
        }
    }

    struct DropGuard<T, const N: usize> {
        items: MaybeUninit<[T; N]>,
        len: usize,
    }

    impl<T, const N: usize> DropGuard<T, N> {
        pub fn new() -> Self {
            Self {
                items: MaybeUninit::uninit(),
                len: 0,
            }
        }

        pub unsafe fn push_unchecked(&mut self, item: T) {
            (self.items.as_mut_ptr() as *mut T)
                .add(self.len)
                .write(item);
            self.len = self.len.unchecked_add(1);
        }

        pub unsafe fn assume_init(self) -> [T; N] {
            let res = ptr::read(&self.items).assume_init();
            mem::forget(self);
            res
        }
    }

    impl<T, const N: usize> Drop for DropGuard<T, N> {
        fn drop(&mut self) {
            unsafe {
                ptr::drop_in_place(slice::from_raw_parts_mut(self.items.as_mut_ptr(), self.len))
            }
        }
    }
}
