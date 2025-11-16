use dioxus::prelude::*;
use std::ops::{Deref, DerefMut};

pub struct SignalVec<T, U = UnsyncStorage>
where
T: 'static,
U: Storage<SignalData<Vec<T>>> + Storage<SignalData<Option<usize>>>,
{
vec: Signal<Vec<T>, U>,
curr: Signal<Option<usize>, U>,
}

pub fn use_signal_vec<T>() -> SignalVec<T> {
use_hook(SignalVec::<T>::default)
}

impl<T, U> Deref for SignalVec<T, U>
where
T: 'static,
U: Storage<SignalData<Vec<T>>> + Storage<SignalData<Option<usize>>>,
{
type Target = Signal<Vec<T>, U>;

fn deref(&self) -> &Self::Target {
    &self.vec
}
}

impl<T, U> DerefMut for SignalVec<T, U>
where
T: 'static,
U: Storage<SignalData<Vec<T>>> + Storage<SignalData<Option<usize>>>,
{
fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.vec
}
}

impl<T, U> PartialEq for SignalVec<T, U>
where
T: 'static,
U: Storage<SignalData<Vec<T>>> + Storage<SignalData<Option<usize>>>,
{
fn eq(&self, other: &Self) -> bool {
    self.vec == other.vec && self.curr == other.curr
}
}

impl<T, U> Clone for SignalVec<T, U>
where
T: 'static,
U: Storage<SignalData<Vec<T>>> + Storage<SignalData<Option<usize>>>,
{
fn clone(&self) -> Self {
    Self {
        vec: self.vec.clone(),
        curr: self.curr.clone(),
    }
}
}

impl<T, U> Copy for SignalVec<T, U>
where
T: 'static,
U: Storage<SignalData<Vec<T>>> + Storage<SignalData<Option<usize>>>,
{
}

impl<T, U> Default for SignalVec<T, U>
where
T: 'static,
U: Storage<SignalData<Vec<T>>> + Storage<SignalData<Option<usize>>>,
{
fn default() -> Self {
    Self {
        vec: Default::default(),
        curr: Default::default(),
    }
}
}

impl<T, U> SignalVec<T, U>
where
T: 'static,
U: Storage<SignalData<Vec<T>>> + Storage<SignalData<Option<usize>>>,
{
pub fn selection_idx(&self) -> Option<usize> {
    (self.curr)()
}

pub fn selection(&self) -> Option<T>
where
    T: Clone,
{
    self.selection_idx()
        .and_then(|idx| self.vec.read().get(idx).cloned())
}

pub fn set_selection(&mut self, val: Option<usize>) {
    self.curr.set(val);
}

pub fn push(&mut self, item: T) {
    let mut writer = self.vec.write();
    self.curr.set(Some(writer.len()));
    writer.push(item);
}

pub fn remove(&mut self, index: usize) {
    let mut writer = self.vec.write();
    writer.remove(index);
    if let Some(curr) = (self.curr)() {
        if curr >= index {
            self.curr.set(curr.checked_sub(1).or_else(|| {
                if writer.len() > 0 {
                    Some(0)
                } else {
                    None
                }
            }));
        }
    }
}
}
