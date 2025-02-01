use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    color_map::ColorWeight, color_handle::{self, Handle}, slice_owner::Slice, verboser::Verboser, Image
};
    
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Item {
    pub color_idx: usize,
    pub cap: usize,
}

impl Item {
    pub fn new(color_idx: usize, cap: usize) -> Self {
        Item { color_idx, cap }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Group<C = Vec<Item>>(C);

impl<C> Group<C> {
    pub fn new(items: C) -> Self {
        Group(items)
    }
}   
impl<C> Deref for Group<C>{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for Group<C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Config<G = Group<Vec<Item>>, C = Vec<Item>> {
    groups: G,
    _panthom: std::marker::PhantomData<C>,
}

impl<G, C> Config<G, C>{
    pub fn new(groups: G) -> Self{
        Self{
            groups,
            _panthom: std::marker::PhantomData,
        }
    }
}

impl<G, C> Deref for Config<G, C>{
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

impl<G, C> DerefMut for Config<G, C>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.groups
    }
}

impl<G, C> From<&Handle<> for Config<G, C>{
    fn from(value: &Selector) -> Self {
        Config::new(value.iter().map(|group| {
            Group::new(group.iter().map(|item| {
                Item::new(item.color_idx(), item.cap())
            }).collect())
        }).collect())   
    }
}

unsafe impl<S, G, C> color_handle::Builder<S> for Config<G, C>
where
    G: AsRef<[Group<C>]>,
    C: AsRef<[Item]>,
{
    fn build_line_selector<L, Sl: ?Sized + Slice<Item = ColorWeight<L, S>>>(
        &self,
        image: &Image<S>,
        weights: &Sl,
        verboser: &mut impl Verboser,
    ) -> Result<Handle<Sl::Map<color_handle::Group>>, color_handle::Error> {
        self.groups
            .as_ref()
            .iter()
            .map(|group| {
                group
                    .0
                    .as_ref()
                    .iter()
                    .map(|item| {
                        if item.color_idx >= weights.len() {
                            Err(color_handle::Error)
                        } else {
                            Ok(color_handle::Item::new(item.color_idx, 0, item.cap))
                        }
                    })
                    .collect()
            })
    }
}

impl From<Item> for Item {
    fn from(value: Item) -> Self {
        Item::new(value.color_idx, 0, value.cap)
    }
}
