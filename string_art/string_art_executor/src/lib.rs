//! Execution primitives shared by the computation core and its backends.

/// A unit of work that can be run against one mutable buffer.
pub trait Work<T> {
    fn run(&self, index: usize, buffer: &mut T);
}

impl<T, F> Work<T> for F
where
    F: Fn(usize, &mut T),
{
    #[inline]
    fn run(&self, index: usize, buffer: &mut T) {
        self(index, buffer);
    }
}

/// Executes typed work over independent mutable buffers.
///
/// The bounds required to cross threads belong to each backend implementation,
/// not to the core or to this trait.
pub trait Executor<T, W: Work<T>> {
    fn workers(&self) -> usize;

    fn for_each(&self, buffers: &mut [T], work: W);
}
