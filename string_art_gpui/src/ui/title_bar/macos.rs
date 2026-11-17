//! macOS window chrome.
//!
//! The native traffic lights are kept: the title bar is transparent and the
//! application only reserves space for them. Dragging is app-owned (see
//! `WindowOptions::app_owns_titlebar_drag`), and double-clicking follows the
//! system preference through `Window::titlebar_double_click`.

use gpui::{App, Div, Pixels, Window, px};

/// Left inset reserved for the native traffic lights.
pub const LEFT_INSET: Pixels = px(84.);
pub const RIGHT_INSET: Pixels = px(12.);

pub fn custom_chrome(_window: &Window) -> bool {
    true
}

pub fn left_controls(_window: &mut Window, _cx: &mut App) -> Option<Div> {
    None
}

pub fn right_controls(_window: &mut Window, _cx: &mut App) -> Option<Div> {
    None
}

pub fn resize_handles(_window: &Window) -> Option<Div> {
    None
}
