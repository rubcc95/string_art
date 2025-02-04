use std::ops::{Deref, DerefMut};

use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};

use crate::{
    image::Image,
    slice::{Slice, SliceOwner},
    verboser::Verboser,
    Float,
};

use super::MapBuilder;

#[derive(Clone, Serialize, Deserialize)]
pub struct Auto<
    G = Vec<Group<f32>>,
    //C = Vec<usize>,
> {
    pub groups: G,
    pub threads: usize,
}

impl<G> Auto<G> {
    #[inline]
    pub fn new(groups: G, threads: usize) -> Self {
        Auto { groups, threads }
    }
}

unsafe impl<'a, S, G, C: 'a, T: Float> super::Builder<'a, S> for Auto<G>
where
    S: Float,
    G: SliceOwner<'a, Item = Group<T, C>>,
    C: SliceOwner<'a, Item = usize>,
    usize: AsPrimitive<S>,
    T: AsPrimitive<S>,
{
    type Groups = G::Map<'a, super::Group<C::Map<'a, super::Item>>>;

    fn build_handle<L, Sl: ?Sized + Slice<'a, Item = MapBuilder<L, S>>>(
        self,
        image: &Image<S>,
        weights: &Sl,
        _: &mut impl Verboser,
    ) -> Result<super::Config<Self::Groups>, super::LALAError> {
        let mut weights: <Sl as Slice<'_>>::Map<'_, AutoLineDitherCounter<'_, L, S>> =
            weights.map(|counter: &MapBuilder<L, S>| AutoLineDitherCounter {
                color: counter,
                weight: S::ZERO,
            });
            println!("Runnnnn");
        for group in self.groups.as_slice().raw_slice() {
            for &index in group.colors.as_slice().raw_slice() {
                match weights.as_mut_slice().get_mut(index) {
                    Some(weight) => weight.weight += group.weight.as_(),
                    None => return Err(super::LALAError),
                }
            }
        }
        let pixel_count = image.pixels().len().as_();
        let threads = self.threads.as_();

        let res = self.groups.map(|group| {
            super::Group::new(group.colors.map(|idx| {
                let weight = unsafe { weights.as_slice().get_unchecked(idx) };
                let prop =
                    (AsPrimitive::<S>::as_(group.weight) * weight.color.count.as_()) / (pixel_count * weight.weight);
                super::Item::new(idx, (threads * prop).to_usize().unwrap())
            }))
        });
        Ok(super::Config::new(res))
    }

    // fn build_handle<L, Sl: ?Sized + Slice<'a, Item = ColorWeight<L, S>>>(
    //     &self,
    //     image: &Image<S>,
    //     weights: &Sl,
    //     verboser: &mut impl Verboser,
    // ) -> Result<color_super::Handle<Self::Groups>, color_super::Error> {
    //     self.build_handle(image, weights, verboser)
    // }
}

// unsafe impl<S, T, G, C> color_handle::Builder<T> for AutoLineConfig<S, G, C>
// where
//     usize: AsPrimitive<T>,
//     S: AsPrimitive<T>,
//     T: Float,
//     G: AsRef<[AutoLineGroupConfig<S, C>]>,
//     C: AsRef<[usize]>,
// {
//     fn build_line_selector<L>(
//         &self,
//         image: &Image<T>,
//         weights: &[ColorWeight<L, T>],
//         verboser: &mut impl Verboser
//     ) -> Result<Handle, color_handle::Error> {
//         self.bake(image, weights.iter(), verboser)
//     }
// }

impl<G> Deref for Auto<G> {
    type Target = G;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

impl<S> DerefMut for Auto<S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.groups
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Group<S, C = Vec<usize>> {
    pub colors: C,
    pub weight: S,
}

impl<S, C> Group<S, C> {
    #[inline]
    pub fn new(colors: C, weight: S) -> Self {
        Group { colors, weight }
    }
}

impl<S> Deref for Group<S> {
    type Target = Vec<usize>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.colors
    }
}

impl<S> DerefMut for Group<S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.colors
    }
}

struct AutoLineDitherCounter<'a, L, S> {
    color: &'a MapBuilder<L, S>,
    weight: S,
}
