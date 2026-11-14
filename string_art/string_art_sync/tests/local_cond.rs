//! Without the `threads` feature no backend crosses threads, so `CondSend` and
//! `CondSync` are empty: the single-threaded computation has to accept **any**
//! type, `Rc` included.
//!
//! With `threads` enabled this file becomes empty: that same code stops
//! compiling, which is exactly what should happen.

#![cfg(not(feature = "multithreaded"))]

use std::rc::Rc;

use string_art_sync::{CondSend, CondSync, Executor, Serial};

fn assert_cond<T: CondSend + CondSync>() {}

#[test]
fn any_type_is_cond_send_and_cond_sync_without_threads() {
    assert_cond::<Rc<()>>();
    assert_cond::<Vec<Rc<()>>>();
    assert_cond::<*const u8>();
}

#[test]
fn serial_accepts_a_buffer_that_is_not_send() {
    let mut bufs: Vec<Option<Rc<u8>>> = vec![None, None];

    Serial.for_each(&mut bufs, |index, slot| {
        *slot = Some(Rc::new(index as u8));
    });

    assert_eq!(
        bufs.iter().map(|b| *b.clone().unwrap()).collect::<Vec<_>>(),
        vec![0, 1]
    );
}

#[test]
fn serial_accepts_a_closure_that_is_not_send() {
    let shared = Rc::new(0u8);
    let mut bufs = [0u8; 2];

    Serial.for_each(&mut bufs, |index, slot| {
        *slot = *shared + index as u8;
    });

    assert_eq!(bufs, [0, 1]);
}
