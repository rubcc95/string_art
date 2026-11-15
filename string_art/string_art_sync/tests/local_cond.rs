//! The work abstraction itself does not impose thread-safety bounds.

use std::rc::Rc;
use string_art_sync::Work;

struct RcWork(Rc<u8>);

impl Work<Rc<u8>> for RcWork {
    fn run(&self, _: usize, buffer: &mut Rc<u8>) {
        *buffer = self.0.clone();
    }
}

#[test]
fn work_accepts_local_captures_and_buffers() {
    let value = Rc::new(7);
    let mut buffer = Rc::new(0);
    RcWork(value.clone()).run(0, &mut buffer);
    assert!(Rc::ptr_eq(&buffer, &value));
}
