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

use super::controls::{self, Glyph};
use crate::ui::{DISABLED_TEXT, TEXT};

pub const LEFT_INSET: Pixels = px(0.);
pub const RIGHT_INSET: Pixels = px(0.);

pub fn custom_chrome(_window: &Window) -> bool {
    true
}

pub fn left_controls(_window: &mut Window, _cx: &mut App) -> Option<Div> {
    None
}

pub fn right_controls(window: &mut Window, cx: &mut App) -> Option<Div> {
    Some(windows_controls(window, cx))
}

pub fn resize_handles(_window: &Window) -> Option<Div> {
    None
}

fn windows_controls(window: &mut Window, cx: &mut App) -> Div {
    div()
        .flex()
        .items_center()
        .h_full()
        .child(windows_control(WindowsControl::Minimize, window, cx))
        .child(windows_control(
            if window.is_maximized() {
                WindowsControl::Restore
            } else {
                WindowsControl::Maximize
            },
            window,
            cx,
        ))
        .child(windows_control(WindowsControl::Close, window, cx))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowsControl {
    Minimize,
    Maximize,
    Restore,
    Close,
}

fn windows_control(control: WindowsControl, window: &mut Window, cx: &mut App) -> Stateful<Div> {
    let supported = window.window_controls();
    let enabled = match control {
        WindowsControl::Minimize => supported.minimize && window.is_minimizable(),
        WindowsControl::Maximize | WindowsControl::Restore => {
            supported.maximize && window.is_resizable()
        }
        WindowsControl::Close => true,
    };

    let (id, shape, area) = match control {
        WindowsControl::Minimize => (
            "window-control-minimize",
            Glyph::Minimize,
            WindowControlArea::Min,
        ),
        WindowsControl::Maximize => (
            "window-control-maximize",
            Glyph::Maximize,
            WindowControlArea::Max,
        ),
        WindowsControl::Restore => (
            "window-control-restore",
            Glyph::Restore,
            WindowControlArea::Max,
        ),
        WindowsControl::Close => (
            "window-control-close",
            Glyph::Close,
            WindowControlArea::Close,
        ),
    };

    let is_close = control == WindowsControl::Close;
    let hover_background = if is_close { 0xc42b1c } else { 0xffffff14 };

    let hover = controls::use_hover(format!("window-control-hover-{id}"), window, cx);
    let fill = controls::hover_fill(
        format!("window-control-fill-{id}"),
        hover.get(cx),
        rgb(hover_background),
        div().absolute().top_0().left_0().size_full(),
    );
    let icon = if is_close {
        controls::close_icon()
    } else {
        let color = if enabled { TEXT } else { DISABLED_TEXT };
        controls::glyph(shape, rgb(color))
    };

    div()
        .id(id)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .occlude()
        .w(px(36.))
        .h_full()
        .window_control_area(area)
        .when(is_close, |button| button.group("window-control-close"))
        .on_hover(hover.listener())
        .child(fill)
        .child(icon)
}
