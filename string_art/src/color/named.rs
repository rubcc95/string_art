use core::ops::{Deref, DerefMut};

use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};

use crate::{
    color::{self, mapping, Rgb},
    Float,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Named<S = u8> {
    pub name: String,
    pub value: Rgb<S>,
}

impl<S> Named<S> {
    pub fn new(name: String, value: Rgb<S>) -> Self {
        Self { name, value }
    }
}

impl<L, S> From<mapping::State<L, S>> for Named<S> {
    fn from(value: mapping::State<L, S>) -> Self {
        value.color
    }
}
 
impl<S> Deref for Named<S> {
    type Target = Rgb<S>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<S> DerefMut for Named<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<S: Float> From<Named> for Named<S>
where
    u8: AsPrimitive<S>,
{
    fn from(value: Named) -> Self {
        Self {
            name: value.name,
            value: color::AsRgb::as_rgb(&value.value),
        }
    }
}
