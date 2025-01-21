use crate::Float;

pub trait Darkness<S>: Send + Sync {
    fn compute(&self, weight: S) -> S;
}

#[derive(Clone, Copy)]
pub struct FlatDarkness<S>(pub S);

impl<T: Float> Darkness<T> for FlatDarkness<T> {
    fn compute(&self, weight: T) -> T {
        (weight - self.0).max(T::ZERO)
    }
}

#[derive(Clone, Copy)]
pub struct PercentageDarkness<S>(pub S);

impl<S: Float> Darkness<S> for PercentageDarkness<S> {
    fn compute(&self, weight: S) -> S {
        self.0 * weight
    }
}