use string_art_sync::{Executor, Work, Workers};

/// Single-threaded executor. It deliberately accepts non-Send and non-Sync work.
#[derive(Clone, Copy, Debug, Default)]
pub struct SerialExec;

impl Workers for SerialExec {
    fn workers(&self) -> usize {
        1
    }
}

impl<T, W: Work<T>> Executor<T, W> for SerialExec {
    fn for_each(&self, buffers: &mut [T], work: W) {
        for (index, buffer) in buffers.iter_mut().enumerate() {
            work.run(index, buffer);
        }
    }
}
