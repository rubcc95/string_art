//! Centered placeholder used when a screen has nothing to show yet.

use gpui::{Div, SharedString, div, prelude::*, rgb};

use super::{MUTED, TEXT};

pub fn empty_state(title: impl Into<SharedString>, message: impl Into<SharedString>) -> Div {
    let title = title.into();
    let message = message.into();
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .p_8()
        .child(div().text_lg().text_color(rgb(TEXT)).child(title))
        .child(div().text_sm().text_color(rgb(MUTED)).child(message))
}
