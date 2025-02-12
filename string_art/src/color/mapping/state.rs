use std::ops::{Deref, DerefMut};

use num_traits::AsPrimitive;

use crate::{color, Float};

#[derive(Clone)]
pub struct State<I, L, S = u8> {
    pub color: color::Named<S>,
    pub nail: I,
    pub link: L,
}

impl<I, L, S> Deref for State<I, L, S> {
    type Target = color::Named<S>;

    fn deref(&self) -> &Self::Target {
        &self.color
    }
}

impl<I, L, S> DerefMut for State<I, L, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.color
    }
}

impl<I: Default, L: Default, S> From<color::Named<S>> for State<I, L, S>{
    fn from(color: color::Named<S>) -> Self {
        Self { color, nail: Default::default(), link: Default::default() }
    }
}

impl<I, L, S> From<color::Map<I, L, S>> for State<I, L> {
    fn from(value: color::Map<I, L, S>) -> Self {
        value.state
    }
}

impl<I, L, S: Float> From<State<I, L>> for State<I, L, S>
where
    u8: AsPrimitive<S>,
{
    fn from(value: State<I, L>) -> Self {
        Self {
            color: value.color.into(),
            nail: value.nail,
            link: value.link,
        }
    }
}

impl<I, L, S> State<I, L, S> {
    pub fn new(color: color::Named<S>, nail: I, link: L) -> Self {
        Self { color, nail, link }
    }
}
