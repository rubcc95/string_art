//! Window chrome for targets without native window decorations, the web build
//! among them.
//!
//! There is nothing to minimize, maximize, drag or close: the title bar is only
//! a plain toolbar. This module exists so the shared UI code compiles unchanged
//! when no desktop platform module is selected.

use gpui::{App, Div, Pixels, Window, px};

pub const LEFT_INSET: Pixels = px(12.);
pub const RIGHT_INSET: Pixels = px(12.);

pub fn custom_chrome(_window: &Window) -> bool {
    false
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
