use string_art_sync::Executor;

/// Its own rayon pool, with a fixed number of threads.
///
/// Having the pool here (and not rayon's global one) makes it possible to
/// choose the number of threads at runtime and measure it without touching
/// environment variables.
#[derive(Clone, Copy, Debug, Default)]
pub struct SerialExec;

impl Executor for SerialExec {
    fn workers(&self) -> usize {
        1
    }

    fn for_each<T, F>(&self, bufs: &mut [T], f: F)
    where
        F: Fn(usize, &mut T),
    {
        f(0, &mut bufs[0])
    }
}
