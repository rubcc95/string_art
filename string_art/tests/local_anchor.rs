//! The core pipeline accepts anchors that are not Send or Sync.

use std::rc::Rc;

use string_art::computation::Pipeline;
use string_art::{Frac, Monocolor};

#[derive(Clone, Default)]
#[allow(dead_code)]
struct LocalAnchor(Rc<()>);

fn assert_pipeline<P: Pipeline<Anchor = LocalAnchor>>() {}

#[test]
fn pipeline_accepts_a_non_send_anchor() {
    assert_pipeline::<Monocolor<LocalAnchor>>();
    let monocolor = Monocolor::<LocalAnchor>::new(LocalAnchor::default(), vec![Frac::ZERO; 4], 1);
    assert_eq!(monocolor.weights().len(), 4);
}
