use crate::num::Frac;

pub struct ColorMap<A, C> {
    pub anchor: A,
    pub color: C,
    pub weights: Vec<Frac>,
}
