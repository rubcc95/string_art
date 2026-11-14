//! Sin la feature `threads` ningún backend cruza hilos, así que `CondSend` y
//! `CondSync` están vacíos: el cálculo monohilo tiene que aceptar **cualquier**
//! tipo, `Rc` incluido.
//!
//! Con `threads` encendida este fichero queda vacío: ese mismo código deja de
//! compilar, que es exactamente lo que debe pasar.

#![cfg(not(feature = "multithreaded"))]

use std::rc::Rc;

use string_art_sync::{CondSend, CondSync, Executor, Serial};

fn assert_cond<T: CondSend + CondSync>() {}

#[test]
fn cualquier_tipo_es_cond_send_y_cond_sync_sin_hilos() {
    assert_cond::<Rc<()>>();
    assert_cond::<Vec<Rc<()>>>();
    assert_cond::<*const u8>();
}

#[test]
fn serial_acepta_un_buffer_que_no_es_send() {
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
fn serial_acepta_un_cierre_que_no_es_send() {
    let shared = Rc::new(0u8);
    let mut bufs = [0u8; 2];

    Serial.for_each(&mut bufs, |index, slot| {
        *slot = *shared + index as u8;
    });

    assert_eq!(bufs, [0, 1]);
}
