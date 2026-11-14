//! With the `threads` feature disabled there is no backend that crosses
//! threads, so `CondSend`/`CondSync` are empty: an `Anchor` with an `Rc` inside
//! must still work.
//!
//! With `threads` enabled this file is intentionally left empty: that same
//! code would stop compiling, because then `CondSend` is `Send` for real.

#![cfg(not(feature = "multithreaded"))]

use std::rc::Rc;

use string_art::Frac;
use string_art::Monocolor;
use string_art::computation::Pipeline;
use string_art_sync::{CondSend, CondSync};

/// Anchor that cannot cross threads: it holds an `Rc`.
#[derive(Clone, Default)]
#[allow(dead_code)]
struct LocalAnchor(Rc<()>);

fn assert_cond<T: CondSend + CondSync>() {}

fn assert_pipeline<P: Pipeline<Anchor = LocalAnchor>>() {}

#[test]
fn any_type_is_cond_send_without_threads() {
    assert_cond::<Rc<()>>();
    assert_cond::<LocalAnchor>();
}

#[test]
fn the_pipeline_accepts_an_anchor_that_is_not_send() {
    assert_pipeline::<Monocolor<LocalAnchor>>();

    let monocolor = Monocolor::<LocalAnchor>::new(LocalAnchor::default(), vec![Frac::ZERO; 4], 1);
    assert_eq!(monocolor.weights().len(), 4);
}
