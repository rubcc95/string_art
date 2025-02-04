use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    image::Image,
    slice::{Slice, SliceOwner},
    verboser::Verboser,
};

use super::MapBuilder;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Item {
    pub color_idx: usize,
    pub cap: usize,
}

impl Item {
    #[inline]
    pub fn new(color_idx: usize, cap: usize) -> Self {
        Item { color_idx, cap }
    }
}

impl From<super::Item> for Item{
    #[inline]
    fn from(value: super::Item) -> Self {
        Item::new(value.color_idx, value.cap)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Group<C = Vec<Item>>(C);

impl<C> Group<C> {
    #[inline]
    pub fn new(items: C) -> Self {
        Group(items)
    }
}
impl<C> Deref for Group<C> {
    type Target = C;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for Group<C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Manual<G = Vec<Group<Vec<Item>>>> {
    groups: G,
}

impl<G> Manual<G> {
    #[inline]
    pub fn new(groups: G) -> Self {
        Self {
            groups,
        }
    }
}

impl<G> Deref for Manual<G> {
    type Target = G;
    
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

impl<G> DerefMut for Manual<G> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.groups
    }
}

unsafe impl<'a, S, G, C: 'a> super::Builder<'a, S> for Manual<G>
where
    G: SliceOwner<'a, Item = Group<C>>,
    C: SliceOwner<'a, Item = Item>,
{
    type Groups = G::Map<'a, super::Group<C::Map<'a, super::Item>>>;

    fn build_handle<L, Sl: ?Sized + Slice<'a, Item = MapBuilder<L, S>>>(
        self,
        _: &Image<S>,
        weights: &Sl,
        _: &mut impl Verboser,
    ) -> Result<super::Config<Self::Groups>, super::LALAError> {
        self.groups
            
            .try_map(|group| {
                let b = group.0.try_map(|item| {
                    if item.color_idx >= weights.len() {
                        Err(super::LALAError)
                    } else {
                        Ok(super::Item::new(item.color_idx, item.cap))
                    }
                }).map(super::Group::new);
                b
            })
            .map(super::Config::new)
    }
}
