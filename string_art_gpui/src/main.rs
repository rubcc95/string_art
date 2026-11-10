//! La aplicación: ventana de gpui que corre el cálculo sobre el pool de hilos.
//!
//! Nativo: `cargo run -p string_art_gpui`.
//! Web: `cargo build --target wasm32-unknown-unknown ...` y cargar el `.wasm`.

#![cfg_attr(target_family = "wasm", no_main)]

mod exec;

use gpui::{
    App, Bounds, Context, Render, SharedString, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_platform::application;
use string_art_sync::Parallel;

use crate::exec::GpuiExec;

struct View {
    status: SharedString,
}

impl Render for View {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size(px(480.0))
            .justify_center()
            .items_center()
            .bg(rgb(0x1e1e1e))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child("String Art")
            .child(self.status.clone())
    }
}

fn run() {
    application().run(|cx: &mut App| {
        let exec = GpuiExec::new(gpui_platform::background_executor());
        let workers = exec.workers();

        // El reparto va en una tarea de fondo: `for_each` bloquea al que llama y
        // el hilo principal no puede esperar.
        let background = exec.clone();
        cx.background_executor()
            .spawn(async move {
                let mut slots = [0u64; 8];
                background.for_each(&mut slots, |index, slot| {
                    let mut acc = 0u64;
                    for value in 0..200_000u64 {
                        acc = acc.wrapping_add(value ^ index as u64);
                    }
                    *slot = acc;
                });
            })
            .detach();

        let status: SharedString = format!("hilos disponibles: {workers}").into();
        let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| View { status }),
        )
        .unwrap();

        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run();
}
