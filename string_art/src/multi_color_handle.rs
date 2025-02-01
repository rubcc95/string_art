use std::ops::Deref;

use crate::{
    color_map::ColorWeight,
    slice_owner::{Slice, SliceOwner},
    verboser::Verboser,
    ColorConfig, Image,
};

pub unsafe trait Builder<S> {
    fn build_line_selector<L, Sl: ?Sized + Slice<Item = ColorWeight<L, S>>>(
        &self,
        image: &Image<S>,
        weights: &Sl,
        verboser: &mut impl Verboser,
    ) -> Result<Handle<Sl::Map<Group>>, Error>;
}

#[derive(Clone, Copy)]
pub struct Item {
    color_idx: usize,
    count: usize,
    cap: usize,
}

impl Item {
    pub fn new(color_idx: usize, count: usize, cap: usize) -> Self {
        Item {
            color_idx,
            count,
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

pub struct Group<S= Vec<Item>>(S);

// impl<S: SliceOwner<Item = Item>> FromIterator<Item> for Group {
//     fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
//         Group(iter.into_iter().collect())
//     }
// }

impl<'a, S: 'a + SliceOwner<Item = Item>> Group<S> {
    fn select_next(&'a mut self) -> Option<usize> {
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

pub struct Handle<G> {
    groups: G,
    curr: usize,
}

impl<G: SliceOwner<Item = Group<I>>, I> Handle<G> {
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

impl<S> Deref for Handle<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.groups
    }
}

// impl<S> FromIterator<S> for Handle<S> {
//     fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
//         let lines: Vec<_> = iter.into_iter().collect();
//         Handle {
//             curr: lines.len().checked_sub(1).unwrap_or(lines.len()),
//             groups: lines,
//         }
//     }
//     // fn from_iter<T: IntoIterator<Item = Group>>(iter: T) -> Self {
//     //     let lines: Vec<_> = iter.into_iter().collect();
//     //     Handle{
//     //         curr: lines.len().checked_sub(1).unwrap_or(lines.len()),
//     //         groups: lines,

//     //     }
//     // }
// }

#[derive(Debug, thiserror::Error)]
#[error("Invalid group index")]
pub struct Error;
