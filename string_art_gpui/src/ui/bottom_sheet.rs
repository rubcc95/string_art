//! Bottom sheet overlay used by the compact image picker.

use std::rc::Rc;

use gpui::{
    AnyElement, App, MouseButton, SharedString, Window, div, prelude::*, relative, rgb, rgba,
};

use super::{BORDER, MUTED, PANEL, TEXT};

pub fn bottom_sheet(
    id: &'static str,
    title: impl Into<SharedString>,
    content: AnyElement,
    footer: Option<AnyElement>,
    on_close: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let title = title.into();
    let on_close = Rc::new(on_close);

    let scrim_close = on_close.clone();
    let button_close = on_close.clone();

    div()
        .id(id)
        .absolute()
        .inset_0()
        .child(
            div()
                .id((id, 0u32))
                .absolute()
                .inset_0()
                .bg(rgba(0x000000a6))
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    scrim_close(window, cx)
                }),
        )
        .child(
            div()
                .id((id, 1u32))
                .absolute()
                .bottom_0()
                .left_0()
                .right_0()
                .max_h(relative(0.75))
                .flex()
                .flex_col()
                .rounded_t_xl()
                .border_t_1()
                .border_color(rgb(BORDER))
                .bg(rgb(PANEL))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .border_b_1()
                        .border_color(rgb(BORDER))
                        .child(div().text_sm().text_color(rgb(TEXT)).child(title))
                        .child(
                            div()
                                .id((id, 2u32))
                                .px_2()
                                .rounded_md()
                                .text_sm()
                                .text_color(rgb(MUTED))
                                .cursor_pointer()
                                .hover(|style| style.text_color(rgb(TEXT)))
                                .on_click(move |_, window, cx| button_close(window, cx))
                                .child("Cerrar"),
                        ),
                )
                .child(content)
                .when_some(footer, |sheet, footer| sheet.child(footer)),
        )
}
