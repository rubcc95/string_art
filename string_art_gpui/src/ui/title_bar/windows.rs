//! Windows window chrome.
//!
//! The native caption is hidden (`appears_transparent`) so the application can
//! draw its own title bar, but the caption buttons are still handled by the
//! platform: registering them as window control areas makes Windows treat them
//! as `HTMINBUTTON`, `HTMAXBUTTON` and `HTCLOSE`, which gives native hover
//! behaviour, Snap Layouts, dragging and double-click-to-maximize for free.

use gpui::{
    App, Div, InteractiveElement, ParentElement, Pixels, Stateful, StatefulInteractiveElement,
    Styled, Window, WindowControlArea, div, prelude::*, px, rgb,
};

use crate::ui::{DISABLED_TEXT, TEXT};

pub const LEFT_INSET: Pixels = px(0.);
pub const RIGHT_INSET: Pixels = px(0.);

pub fn custom_chrome(_window: &Window) -> bool {
    true
}

pub fn left_controls(_window: &Window, _cx: &App) -> Option<Div> {
    None
}

pub fn right_controls(window: &Window, _cx: &App) -> Option<Div> {
    Some(windows_controls(window))
}

pub fn resize_handles(_window: &Window) -> Option<Div> {
    None
}

fn windows_controls(window: &Window) -> Div {
    div()
        .flex()
        .items_center()
        .h_full()
        .font_family("Segoe Fluent Icons")
        .text_size(px(10.))
        .child(windows_control(WindowsControl::Minimize, window))
        .child(windows_control(
            if window.is_maximized() {
                WindowsControl::Restore
            } else {
                WindowsControl::Maximize
            },
            window,
        ))
        .child(windows_control(WindowsControl::Close, window))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowsControl {
    Minimize,
    Maximize,
    Restore,
    Close,
}

fn windows_control(control: WindowsControl, window: &Window) -> Stateful<Div> {
    let supported = window.window_controls();
    let enabled = match control {
        WindowsControl::Minimize => supported.minimize && window.is_minimizable(),
        WindowsControl::Maximize | WindowsControl::Restore => {
            supported.maximize && window.is_resizable()
        }
        WindowsControl::Close => true,
    };

    let (id, glyph, area) = match control {
        WindowsControl::Minimize => (
            "window-control-minimize",
            "\u{e921}",
            WindowControlArea::Min,
        ),
        WindowsControl::Maximize => (
            "window-control-maximize",
            "\u{e922}",
            WindowControlArea::Max,
        ),
        WindowsControl::Restore => ("window-control-restore", "\u{e923}", WindowControlArea::Max),
        WindowsControl::Close => ("window-control-close", "\u{e8bb}", WindowControlArea::Close),
    };

    let is_close = control == WindowsControl::Close;
    let hover_background = if is_close { 0xc42b1c } else { 0xffffff14 };

    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .occlude()
        .w(px(36.))
        .h_full()
        .text_color(rgb(if enabled { TEXT } else { DISABLED_TEXT }))
        .window_control_area(area)
        .when(enabled, move |button| {
            button.hover(move |style| {
                style.bg(rgb(hover_background)).text_color(rgb(if is_close {
                    0xffffff
                } else {
                    TEXT
                }))
            })
        })
        .child(glyph)
}
