//! La aplicación: ventana de gpui que mide el cálculo secuencial y en paralelo.

#![cfg_attr(target_family = "wasm", no_main)]

mod bench;
#[cfg(target_family = "wasm")]
mod exec;

use std::borrow::Cow;

use gpui::{
    App, BackgroundExecutor, Bounds, Context, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_platform::application;
use web_time::Instant;

#[cfg(target_family = "wasm")]
use crate::exec::GpuiExec;

struct View {
    serial_ms: Option<f64>,
    parallel_ms: Option<f64>,
    running: bool,
}

impl View {
    fn start(&mut self, parallel: bool, cx: &mut Context<Self>) {
        if self.running {
            return;
        }
        self.running = true;
        cx.notify();

        let exec = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let ms = exec
                .clone()
                .spawn(async move {
                    let start = Instant::now();
                    if parallel {
                        run_parallel(exec);
                    } else {
                        bench::run(string_art::sync::Serial);
                    }
                    start.elapsed().as_secs_f64() * 1000.0
                })
                .await;

            let _ = this.update(cx, |this, cx| {
                if parallel {
                    this.parallel_ms = Some(ms);
                } else {
                    this.serial_ms = Some(ms);
                }
                this.running = false;
                cx.notify();
            });
        })
        .detach();
    }
}

#[cfg(target_family = "wasm")]
fn run_parallel(exec: BackgroundExecutor) {
    bench::run(GpuiExec::new(exec));
}

#[cfg(not(target_family = "wasm"))]
fn run_parallel(_: BackgroundExecutor) {
    bench::run(string_art::sync::RayonExec::auto());
}

impl Render for View {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .id("serial")
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x2563eb))
                            .cursor_pointer()
                            .child("Secuencial")
                            .on_click(cx.listener(|this, _, _, cx| this.start(false, cx))),
                    )
                    .child(
                        div()
                            .id("parallel")
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x2563eb))
                            .cursor_pointer()
                            .child("Paralelo")
                            .on_click(cx.listener(|this, _, _, cx| this.start(true, cx))),
                    ),
            )
            .child(format!(
                "Secuencial: {}",
                self.serial_ms
                    .map_or_else(|| "—".to_string(), |ms| format!("{ms:.1} ms"))
            ))
            .child(format!(
                "Paralelo: {}",
                self.parallel_ms
                    .map_or_else(|| "—".to_string(), |ms| format!("{ms:.1} ms"))
            ))
    }
}

fn run() {
    application().run(|cx: &mut App| {
        cx.text_system()
            .add_fonts(vec![Cow::Borrowed(include_bytes!(
                "../assets/fonts/IBMPlexSans-Regular.ttf"
            ))])
            .expect("failed to load application fonts");

        let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| View {
                    serial_ms: None,
                    parallel_ms: None,
                    running: false,
                })
            },
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
