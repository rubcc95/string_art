//! The generic serial executor contract permits local, non-Send work.

use std::rc::Rc;
use string_art_sync::{Executor, Work};

struct Serial;

impl<T, W: Work<T>> Executor<T, W> for Serial {
    fn for_each(&self, buffers: &mut [T], work: W) {
        if let Some(buffer) = buffers.first_mut() {
            work.run(0, buffer);
        }
    }
}

struct RcWork(Rc<u8>);

impl Work<Rc<u8>> for RcWork {
    fn run(&self, _: usize, buffer: &mut Rc<u8>) {
        *buffer = self.0.clone();
    }
}

#[test]
fn serial_contract_accepts_non_send_buffers_and_work() {
    let value = Rc::new(7);
    let mut buffers = [Rc::new(0)];
    Serial.for_each(&mut buffers, RcWork(value.clone()));
    assert_eq!(*buffers[0], 7);
    assert!(Rc::ptr_eq(&buffers[0], &value));
}

#[test]
fn serial_contract_accepts_a_closure_capturing_rc() {
    let shared = Rc::new(5u8);
    let mut buffers = [0u8];
    Serial.for_each(&mut buffers, |_, slot| *slot = *shared);
    assert_eq!(buffers, [5]);
}
