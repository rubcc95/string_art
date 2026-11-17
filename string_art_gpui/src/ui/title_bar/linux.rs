//! Linux window chrome.
//!
//! On Wayland, and on X11 when a compositor is available, the window uses
//! client-side decorations: the application draws the controls and the resize
//! handles. The button layout comes from the desktop environment (GNOME's
//! `gtk-decoration-layout`).
//!
//! Whether the chrome is actually drawn is a runtime decision: the compositor
//! may still grant server-side decorations, in which case the desktop draws the
//! window frame and the application does not.

use gpui::{
    AnyElement, App, CursorStyle, Decorations, Div, InteractiveElement, IntoElement,
    MAX_BUTTONS_PER_SIDE, MouseButton, ParentElement, Pixels, ResizeEdge, Stateful,
    StatefulInteractiveElement, Styled, Tiling, Window, WindowButton, WindowButtonLayout, div,
    prelude::FluentBuilder as _, px, rgb,
};

use super::controls::{self, Glyph};
use crate::ui::TEXT;

pub const LEFT_INSET: Pixels = px(12.);
pub const RIGHT_INSET: Pixels = px(12.);

/// Hover background of the window controls.
const CONTROL_HOVER: u32 = 0x2a2b31;
/// Hover background of the close button, following the desktop convention.
const CLOSE_HOVER: u32 = 0xc42b1c;

/// Thickness of the invisible strips used to resize a client-decorated window.
const RESIZE_EDGE_THICKNESS: Pixels = px(6.);
const RESIZE_CORNER_SIZE: Pixels = px(12.);

const FALLBACK_RIGHT: [Option<WindowButton>; MAX_BUTTONS_PER_SIDE] = [
    Some(WindowButton::Minimize),
    Some(WindowButton::Maximize),
    Some(WindowButton::Close),
];

pub fn custom_chrome(window: &Window) -> bool {
    matches!(window.window_decorations(), Decorations::Client { .. })
}

pub fn left_controls(window: &mut Window, cx: &mut App) -> Option<Div> {
    if !custom_chrome(window) {
        return None;
    }
    linux_controls(button_layout(cx).left, window, cx)
}

pub fn right_controls(window: &mut Window, cx: &mut App) -> Option<Div> {
    if !custom_chrome(window) {
        return None;
    }
    linux_controls(button_layout(cx).right, window, cx)
}

/// Resize handles are only needed with client-side decorations, and only when
/// the window can actually be resized in that direction. Tiled, maximized and
/// fullscreen edges are skipped so no resize cursor is shown there.
pub fn resize_handles(window: &Window) -> Option<Div> {
    match window.window_decorations() {
        Decorations::Client { tiling } if !window.is_maximized() && !window.is_fullscreen() => {
            Some(resize_edges(tiling))
        }
        _ => None,
    }
}

/// The button layout configured by the desktop environment, with a GNOME-like
/// fallback when the platform does not report one.
fn button_layout(cx: &App) -> WindowButtonLayout {
    cx.button_layout().unwrap_or(WindowButtonLayout {
        left: [None; MAX_BUTTONS_PER_SIDE],
        right: FALLBACK_RIGHT,
    })
}

fn linux_controls(
    buttons: [Option<WindowButton>; MAX_BUTTONS_PER_SIDE],
    window: &mut Window,
    cx: &mut App,
) -> Option<Div> {
    let supported = window.window_controls();
    let is_maximized = window.is_maximized();

    let elements: Vec<AnyElement> = buttons
        .into_iter()
        .flatten()
        .filter(|button| match button {
            WindowButton::Minimize => supported.minimize && window.is_minimizable(),
            WindowButton::Maximize => supported.maximize && window.is_resizable(),
            WindowButton::Close => true,
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|button| linux_control(button, is_maximized, window, cx).into_any_element())
        .collect();

    if elements.is_empty() {
        return None;
    }

    Some(
        div()
            .flex()
            .items_center()
            .h_full()
            .gap_1()
            .children(elements),
    )
}

fn linux_control(
    button: WindowButton,
    is_maximized: bool,
    window: &mut Window,
    cx: &mut App,
) -> Stateful<Div> {
    let id = match button {
        WindowButton::Minimize => "window-control-minimize",
        WindowButton::Maximize => "window-control-maximize",
        WindowButton::Close => "window-control-close",
    };
    let is_close = button == WindowButton::Close;
    let hover_background = if is_close { CLOSE_HOVER } else { CONTROL_HOVER };

    let hover = controls::use_hover(format!("window-control-hover-{id}"), window, cx);
    let fill = controls::hover_fill(
        format!("window-control-fill-{id}"),
        hover.get(cx),
        rgb(hover_background),
        div().absolute().top_0().left_0().size_full().rounded_full(),
    );

    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .size(px(32.))
        .cursor_pointer()
        .when(is_close, |button| button.group("window-control-close"))
        .on_hover(hover.listener())
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(move |_, window, cx| {
            cx.stop_propagation();
            match button {
                WindowButton::Minimize => window.minimize_window(),
                WindowButton::Maximize => window.zoom_window(),
                WindowButton::Close => window.remove_window(),
            }
        })
        .child(
            div()
                .relative()
                .size(px(20.))
                .flex()
                .items_center()
                .justify_center()
                .child(fill)
                .child(linux_icon(button, is_maximized)),
        )
}

fn linux_icon(button: WindowButton, is_maximized: bool) -> AnyElement {
    match button {
        WindowButton::Minimize => controls::glyph(Glyph::Minimize, rgb(TEXT)),
        WindowButton::Maximize if is_maximized => controls::glyph(Glyph::Restore, rgb(TEXT)),
        WindowButton::Maximize => controls::glyph(Glyph::Maximize, rgb(TEXT)),
        WindowButton::Close => controls::close_icon(),
    }
}

/// Invisible strips along the window edges and corners that start a resize
/// drag. Needed where the compositor no longer provides the window frame.
fn resize_edges(tiling: Tiling) -> Div {
    let mut edges = div().absolute().top_0().left_0().size_full();

    if !tiling.top {
        edges = edges.child(edge_strip(ResizeEdge::Top, CursorStyle::ResizeUpDown));
    }
    if !tiling.bottom {
        edges = edges.child(edge_strip(ResizeEdge::Bottom, CursorStyle::ResizeUpDown));
    }
    if !tiling.left {
        edges = edges.child(edge_strip(ResizeEdge::Left, CursorStyle::ResizeLeftRight));
    }
    if !tiling.right {
        edges = edges.child(edge_strip(ResizeEdge::Right, CursorStyle::ResizeLeftRight));
    }

    if !tiling.top && !tiling.left {
        edges = edges.child(resize_corner(
            ResizeEdge::TopLeft,
            CursorStyle::ResizeUpLeftDownRight,
            div().top_0().left_0(),
        ));
    }
    if !tiling.top && !tiling.right {
        edges = edges.child(resize_corner(
            ResizeEdge::TopRight,
            CursorStyle::ResizeUpRightDownLeft,
            div().top_0().right_0(),
        ));
    }
    if !tiling.bottom && !tiling.left {
        edges = edges.child(resize_corner(
            ResizeEdge::BottomLeft,
            CursorStyle::ResizeUpRightDownLeft,
            div().bottom_0().left_0(),
        ));
    }
    if !tiling.bottom && !tiling.right {
        edges = edges.child(resize_corner(
            ResizeEdge::BottomRight,
            CursorStyle::ResizeUpLeftDownRight,
            div().bottom_0().right_0(),
        ));
    }

    edges
}

/// Builds one resize strip: top/bottom span the width, left/right the height.
fn edge_strip(edge: ResizeEdge, cursor: CursorStyle) -> Div {
    let strip = div().absolute();
    let strip = match edge {
        ResizeEdge::Top => strip.top_0().left_0().right_0().h(RESIZE_EDGE_THICKNESS),
        ResizeEdge::Bottom => strip.bottom_0().left_0().right_0().h(RESIZE_EDGE_THICKNESS),
        ResizeEdge::Left => strip.top_0().bottom_0().left_0().w(RESIZE_EDGE_THICKNESS),
        ResizeEdge::Right => strip.top_0().bottom_0().right_0().w(RESIZE_EDGE_THICKNESS),
        _ => unreachable!("resize strips only exist on the four sides"),
    };

    strip
        .cursor(cursor)
        .occlude()
        .on_mouse_down(MouseButton::Left, move |_, window, _| {
            window.start_window_resize(edge)
        })
}

fn resize_corner(edge: ResizeEdge, cursor: CursorStyle, position: Div) -> Div {
    position
        .absolute()
        .size(RESIZE_CORNER_SIZE)
        .cursor(cursor)
        .occlude()
        .on_mouse_down(MouseButton::Left, move |_, window, _| {
            window.start_window_resize(edge)
        })
}
