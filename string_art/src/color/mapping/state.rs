use std::ops::{Deref, DerefMut};

use num_traits::AsPrimitive;

use crate::{color, Float};

#[derive(Clone)]
pub struct State<L, S = u8> {
    pub color: color::Named<S>,
    pub nail: usize,
    pub link: L,
}

impl<L, S> Deref for State<L, S> {
    type Target = color::Named<S>;

    fn deref(&self) -> &Self::Target {
        &self.color
    }
}

impl<L, S> DerefMut for State<L, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.color
    }
}

impl<L, S> From<color::Map<L, S>> for State<L> {
    fn from(value: color::Map<L, S>) -> Self {
        value.state
    }
}

impl<L, S: Float> From<State<L>> for State<L, S>
where
    u8: AsPrimitive<S>,
{
    fn from(value: State<L>) -> Self {
        Self {
            color: value.color.into(),
            nail: value.nail,
            link: value.link,
        }
    }
}

impl<L, S> State<L, S> {
    pub fn new(color: color::Named<S>, nail: usize, link: L) -> Self {
        Self { color, nail, link }
    }
}
