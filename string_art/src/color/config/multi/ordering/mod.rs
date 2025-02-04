use crate::{
    color::config::map_builder::Builder as MapBuilder,
    slice::{Slice, SliceOwner},
    verboser::Verboser,
    image::Image,
};
use std::ops::Deref;

pub mod auto;
pub mod manual;

pub unsafe trait Builder<'a, S> {
    type Groups;

    fn build_handle<L, Sl: ?Sized + Slice<'a, Item = MapBuilder<L, S>>>(
        self,
        image: &Image<S>,
        weights: &Sl,
        verboser: &mut impl Verboser,
    ) -> Result<Config<Self::Groups>, LALAError>;
}

#[derive(Clone, Copy)]
pub struct Item {
    color_idx: usize,
    count: usize,
    cap: usize,
}

impl Item {
    pub fn new(color_idx: usize, cap: usize) -> Self {
        Item {
            color_idx,
            count: 0,
            cap,
        }
    }

    pub fn color_idx(&self) -> usize {
        self.color_idx
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

pub struct Group<S = Vec<Item>>(S);

impl<'a, S: SliceOwner<'a, Item = Item>> Group<S> {
    pub fn new(items: S) -> Self {
        Group(items)
    }

    fn select_next(&mut self) -> Option<usize> {
        let mut choice = None;
        let mut best_ratio = 1.0;
        for item in self.0.as_mut_slice().raw_mut_slice().into_iter() {
            let ratio = item.count as f32 / item.cap as f32;
            if ratio < best_ratio {
                best_ratio = ratio;
                choice = Some(item);
            }
        }

        choice.map(|item| {
            item.count += 1;
            item.color_idx
        })
    }
}

impl Deref for Group {
    type Target = [Item];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Config<G> {
    groups: G,
    curr: usize,
}

impl<'a, G: SliceOwner<'a>> Config<G> {
    pub fn new(groups: G) -> Self {
        Config {
            curr: groups.len().checked_sub(1).unwrap_or(groups.len()),
            groups,
        }
    }
}
impl<'a, G: SliceOwner<'a, Item = Group<I>>, I: 'a + SliceOwner<'a, Item = Item>> Config<G> {
    pub(crate) fn select_next(&mut self) -> Option<usize> {
        while let Some(last) = self.groups.as_mut_slice().get_mut(self.curr) {
            if let Some(res) = last.select_next() {
                return Some(res);
            } else {
                self.curr = if self.curr == 0 {
                    self.groups.len()
                } else {
                    unsafe { self.curr.unchecked_sub(1) }
                }
            }
        }

        None
    }
}

impl<S> Deref for Config<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid group index")]
pub struct LALAError;
