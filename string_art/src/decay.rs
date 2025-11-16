use num_traits::{CheckedSub, Zero};
use std::ops::Mul;

pub trait Fn<S>: Send + Sync {
    fn compute(&self, weight: S) -> S;
}

#[derive(Clone, Copy)]
pub struct Flat<S>(pub S);

impl<S: Send + Sync + CheckedSub + Zero> Fn<S> for Flat<S> {
    fn compute(&self, weight: S) -> S {
        weight.checked_sub(&self.0).unwrap_or(S::zero())
    }
}

#[derive(Clone, Copy)]
pub struct Percentage<S>(pub S);

impl<S: Send + Sync> Fn<S> for Percentage<S>
where
    for<'a> &'a S: Mul<S, Output = S>,
{
    fn compute(&self, weight: S) -> S {
        self.0.mul(weight)
    }
}
