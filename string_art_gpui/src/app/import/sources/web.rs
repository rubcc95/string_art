//! Browser-specific image import.
//!
//! GPUI's web backend does not implement native file prompts, and browsers
//! only expose dropped files as `File` objects (never as filesystem paths).
//! Both are handled here with `web-sys`.

use std::cell::RefCell;
use std::path::Path;

use gpui::{AsyncApp, Context, Div, Entity, Stateful, Window};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AddEventListenerOptions, DragEvent, Event, EventTarget, File, HtmlInputElement};

use crate::app::AppView;
use crate::import::{self, SUPPORTED_EXTENSIONS, unsupported_file_message};

const MIME_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/webp",
    "image/gif",
    "image/bmp",
    "image/tiff",
    "image/x-icon",
    "image/x-portable-anymap",
    "image/qoi",
    "image/x-tga",
];

thread_local! {
    static INPUT: RefCell<Option<(HtmlInputElement, Closure<dyn FnMut(Event)>)>> =
        const { RefCell::new(None) };
    /// Keeps the DOM listeners alive for the lifetime of the page.
    static LISTENERS: RefCell<Vec<Closure<dyn FnMut(Event)>>> = const { RefCell::new(Vec::new()) };
}

/// Installs the hidden file input and the window-wide drag & drop listeners.
pub fn init(view: Entity<AppView>, app: AsyncApp) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    let input: HtmlInputElement = document
        .create_element("input")
        .expect("failed to create the file input")
        .dyn_into()
        .expect("the created element is not an input");
    input.set_type("file");
    input.set_accept(&accept_attribute());
    input.set_multiple(false);
    let _ = input.set_attribute(
        "style",
        "position:fixed;left:-10000px;top:0;width:1px;height:1px;opacity:0;",
    );

    let picker_view = view.clone();
    let picker_app = app.clone();
    let input_for_handler = input.clone();
    let handler = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
        // Grab the file before clearing the input: Chromium returns the same
        // live `FileList` object from `files()`, and `set_value("")` empties it
        // in place, so reading it afterwards yields no file at all.
        let file = input_for_handler.files().and_then(|files| files.get(0));
        input_for_handler.set_value("");
        let Some(file) = file else {
            return;
        };
        read_and_import(file, picker_view.clone(), picker_app.clone());
    });
    input.set_onchange(Some(handler.as_ref().unchecked_ref()));

    if let Some(body) = document.body() {
        let _ = body.append_child(&input);
    }
    INPUT.with(|slot| *slot.borrow_mut() = Some((input, handler)));

    install_drop_listeners(&window, view, app);
}

/// Opens the hidden file input.
pub fn pick(_window: &mut Window, _cx: &mut Context<AppView>) {
    INPUT.with(|slot| {
        if let Some((input, _)) = slot.borrow().as_ref() {
            input.set_value("");
            input.click();
        }
    });
}

/// Drops are handled by the window-wide listeners installed in [`init`].
pub fn attach_drop(root: Stateful<Div>, _cx: &mut Context<AppView>) -> Stateful<Div> {
    root
}

fn accept_attribute() -> String {
    let mut parts: Vec<String> = MIME_TYPES.iter().map(|mime| (*mime).to_string()).collect();
    parts.extend(
        SUPPORTED_EXTENSIONS
            .iter()
            .map(|extension| format!(".{extension}")),
    );
    parts.join(",")
}

fn install_drop_listeners(window: &web_sys::Window, view: Entity<AppView>, app: AsyncApp) {
    let target: EventTarget = window.clone().into();

    let enter_view = view.clone();
    let enter_app = app.clone();
    let enter = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
        event.prevent_default();
        event.stop_propagation();
        if let Some(drag) = event.dyn_ref::<DragEvent>() {
            if has_files(drag) {
                set_dragging(&enter_view, &enter_app, true);
            }
        }
    });
    add_listener(&target, "dragenter", enter);

    let over_view = view.clone();
    let over_app = app.clone();
    let over = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
        event.prevent_default();
        event.stop_propagation();
        if let Some(drag) = event.dyn_ref::<DragEvent>() {
            if let Some(data) = drag.data_transfer() {
                data.set_drop_effect("copy");
            }
            if has_files(drag) {
                set_dragging(&over_view, &over_app, true);
            }
        }
    });
    add_listener(&target, "dragover", over);

    let leave_view = view.clone();
    let leave_app = app.clone();
    let leave = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
        let left_window = event
            .dyn_ref::<DragEvent>()
            .and_then(|drag| drag.related_target())
            .is_none();
        if left_window {
            set_dragging(&leave_view, &leave_app, false);
        }
    });
    add_listener(&target, "dragleave", leave);

    let drop_view = view;
    let drop_app = app;
    let drop = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
        event.prevent_default();
        event.stop_propagation();

        let Some(drag) = event.dyn_ref::<DragEvent>() else {
            return;
        };
        let file = drag
            .data_transfer()
            .and_then(|data| data.files())
            .and_then(|files| files.get(0));

        set_dragging(&drop_view, &drop_app, false);

        if let Some(file) = file {
            read_and_import(file, drop_view.clone(), drop_app.clone());
        }
    });
    add_listener(&target, "drop", drop);
}

fn add_listener(target: &EventTarget, event: &str, handler: Closure<dyn FnMut(Event)>) {
    let options = AddEventListenerOptions::new();
    options.set_capture(true);
    target
        .add_event_listener_with_callback_and_add_event_listener_options(
            event,
            handler.as_ref().unchecked_ref(),
            &options,
        )
        .expect("failed to add a browser event listener");
    LISTENERS.with(|listeners| listeners.borrow_mut().push(handler));
}

fn has_files(event: &DragEvent) -> bool {
    let Some(data) = event.data_transfer() else {
        return false;
    };
    let types = data.types();
    (0..types.length()).any(|index| types.get(index).as_string().as_deref() == Some("Files"))
}

fn set_dragging(view: &Entity<AppView>, app: &AsyncApp, dragging: bool) {
    let mut app = app.clone();
    let _ = view.update(&mut app, |this, cx| this.set_dragging(dragging, cx));
}

fn read_and_import(file: File, view: Entity<AppView>, app: AsyncApp) {
    let name = file.name();
    if matches!(import::extension_support(Path::new(&name)), Some(false)) {
        let message = unsupported_file_message(&name);
        let mut app = app;
        let _ = view.update(&mut app, |this, cx| this.import_error(message, cx));
        return;
    }

    wasm_bindgen_futures::spawn_local(async move {
        let mut app = app;
        match read_file(file).await {
            Some((name, bytes)) => {
                let _ = view.update(&mut app, |this, cx| this.import_bytes(name, bytes, cx));
            }
            None => {
                let _ = view.update(&mut app, |this, cx| {
                    this.import_error("The selected file could not be read.".to_string(), cx)
                });
            }
        }
    });
}

async fn read_file(file: File) -> Option<(String, Vec<u8>)> {
    let name = file.name();
    let buffer = JsFuture::from(file.array_buffer()).await.ok()?;
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
    Some((name, bytes))
}
