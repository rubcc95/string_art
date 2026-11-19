//! The String Art application.

#![cfg_attr(target_family = "wasm", no_main)]

mod app;
#[allow(dead_code)]
mod bench;
mod config;
#[cfg(feature = "multithreaded")]
#[allow(dead_code)]
mod exec;
mod i18n;
mod import;
mod ui;

rust_i18n::i18n!("locales", fallback = "en");

use std::borrow::Cow;

use gpui::{
    App, AppContext as _, Bounds, TitlebarOptions, WindowBounds, WindowDecorations, WindowOptions,
    point, px, size,
};
use gpui_platform::application;
use rust_i18n::t;

fn run() {
    application().run(|cx: &mut App| {
        // Registered explicitly: the web build has no system fonts, so the UI
        // font has to travel with the binary.
        cx.text_system()
            .add_fonts(vec![
                Cow::Borrowed(include_bytes!("../assets/fonts/Inter-Regular.ttf")),
                Cow::Borrowed(include_bytes!("../assets/fonts/Inter-Medium.ttf")),
                Cow::Borrowed(include_bytes!("../assets/fonts/Inter-SemiBold.ttf")),
            ])
            .expect("failed to load application fonts");

        // The configuration has to be read before any translated text is
        // produced, and the locale has to be set before the first render.
        let config = config::Config::load();
        i18n::init(&config);

        let restore_bounds = Bounds::centered(None, size(px(1280.), px(800.)), cx);
        let view = cx.new(|cx| app::AppView::new(config, cx));
        let window_view = view.clone();

        cx.open_window(
            WindowOptions {
                // Hide the native caption so the application can draw its own
                // title bar. On macOS the traffic lights stay native and are
                // positioned to match the title bar height.
                titlebar: Some(TitlebarOptions {
                    title: Some(t!("app.title").to_string().into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(18.), px(22.))),
                }),
                window_bounds: Some(WindowBounds::Maximized(restore_bounds)),
                // Client-side decorations on Linux; ignored elsewhere.
                window_decorations: Some(WindowDecorations::Client),
                // macOS: the application moves the window itself.
                app_owns_titlebar_drag: true,
                app_id: Some("string_art".into()),
                window_min_size: Some(size(px(720.), px(520.))),
                ..Default::default()
            },
            move |window, cx| {
                window_view.update(cx, |view, cx| {
                    view.install_window_observers(window, cx);
                });
                window_view
            },
        )
        .unwrap();

        app::init_import(view, cx.to_async());

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
