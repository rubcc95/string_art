//! Compact button with a leading element, used to switch the active item.

use gpui::{
    App, ClickEvent, IntoElement, SharedString, Stateful, Window, div, prelude::*, px, rgb,
};

use super::{MUTED, TEXT};

pub fn switcher_button(
    id: &'static str,
    leading: impl IntoElement,
    label: impl Into<SharedString>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<gpui::Div> {
    let label = label.into();
    div()
        .id(id)
        .flex()
        .items_center()
        .gap_2()
        .max_w(px(200.))
        .px_2()
        .py_1()
        .rounded_lg()
        .cursor_pointer()
        .hover(|style| style.bg(rgb(0x232429)))
        .on_click(on_click)
        .child(leading)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_sm()
                .text_color(rgb(TEXT))
                .child(label),
        )
        .child(div().text_xs().text_color(rgb(MUTED)).child("▾"))
}
