//! Buttons shared by the application screens.

use gpui::{Div, Stateful, div, prelude::*, rgb};

use super::{ACCENT, ACCENT_HOVER, BORDER, DISABLED, DISABLED_TEXT, MUTED, TEXT};

pub fn primary_button(id: &'static str, label: impl AsRef<str>, enabled: bool) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .px_6()
        .py_2()
        .rounded_lg()
        .bg(rgb(if enabled { ACCENT } else { DISABLED }))
        .text_sm()
        .text_color(rgb(if enabled { 0xffffff } else { DISABLED_TEXT }))
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgb(ACCENT_HOVER)))
        })
        .child(label.as_ref().to_string())
}

pub fn secondary_button(id: &'static str, label: impl AsRef<str>, enabled: bool) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .px_4()
        .py_2()
        .rounded_lg()
        .border_1()
        .border_color(rgb(BORDER))
        .text_sm()
        .text_color(rgb(if enabled { MUTED } else { DISABLED_TEXT }))
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0x232429)).text_color(rgb(TEXT)))
        })
        .child(label.as_ref().to_string())
}

pub fn icon_button(id: &'static str, label: impl AsRef<str>, enabled: bool) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .size(gpui::px(40.))
        .rounded_lg()
        .border_1()
        .border_color(rgb(BORDER))
        .text_lg()
        .text_color(rgb(if enabled { MUTED } else { DISABLED_TEXT }))
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0x232429)).text_color(rgb(TEXT)))
        })
        .child(label.as_ref().to_string())
}
