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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use string_art_sync::Executor;

    use super::SerialExec;

    #[test]
    fn accepts_non_send_buffers_and_work() {
        let value = Rc::new(7);
        let mut buffers = [Rc::new(0)];
        SerialExec.for_each(&mut buffers, {
            let value = value.clone();
            move |_, buffer| *buffer = value.clone()
        });
        assert_eq!(*buffers[0], 7);
        assert!(Rc::ptr_eq(&buffers[0], &value));
    }

    #[test]
    fn executes_each_buffer() {
        let mut buffers = [0, 0, 0];
        SerialExec.for_each(&mut buffers, |index, buffer| *buffer = index);
        assert_eq!(buffers, [0, 1, 2]);
    }
}
