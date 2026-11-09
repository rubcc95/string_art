//! Con la feature `threads` apagada no hay ningún backend que cruce hilos, así
//! que `CondSend`/`CondSync` son vacíos: un `Anchor` con `Rc` dentro tiene que
//! valer igual.
//!
//! Con `threads` encendida este fichero queda vacío a propósito: ese mismo
//! código dejaría de compilar, porque entonces `CondSend` es `Send` de verdad.

#![cfg(not(feature = "threads"))]

use std::rc::Rc;

use string_art::computation::Pipeline;
use string_art::monocolor::{Color, Monocolor};
use string_art::sync::{CondSend, CondSync};
use string_art::{ColorMap, Frac};

/// Ancla que no puede cruzar hilos: lleva un `Rc`.
#[derive(Clone, Default)]
struct LocalAnchor(Rc<()>);

fn assert_cond<T: CondSend + CondSync>() {}

fn assert_pipeline<P: Pipeline<Anchor = LocalAnchor>>() {}

#[test]
fn cualquier_tipo_es_cond_send_sin_hilos() {
    assert_cond::<Rc<()>>();
    assert_cond::<LocalAnchor>();
}

#[test]
fn el_pipeline_admite_un_ancla_que_no_es_send() {
    assert_pipeline::<Monocolor<LocalAnchor>>();

    let monocolor = Monocolor::<LocalAnchor> {
        map: ColorMap {
            anchor: LocalAnchor::default(),
            color: Color,
            weights: vec![Frac::ZERO; 4],
        },
        threads: 1,
    };

    assert_eq!(monocolor.map.weights.len(), 4);
}
