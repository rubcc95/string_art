//! Horizontal tab strip used for computations.

use std::rc::Rc;

use gpui::{App, SharedString, Window, div, prelude::*, rgb};

use super::{BG, BORDER, MUTED, PANEL, TAB_BAR_HEIGHT, TEXT};

pub struct TabItem {
    pub label: SharedString,
    pub active: bool,
}

pub fn tab_bar(
    id: &'static str,
    items: Vec<TabItem>,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
    on_new: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let on_select = Rc::new(on_select);
    let tabs = items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let on_select = on_select.clone();
            div()
                .id((id, index))
                .flex_none()
                .h_full()
                .flex()
                .items_center()
                .px_4()
                .rounded_t_md()
                .text_sm()
                .cursor_pointer()
                .when(item.active, |tab| tab.bg(rgb(BG)).text_color(rgb(TEXT)))
                .when(!item.active, |tab| {
                    tab.text_color(rgb(MUTED))
                        .hover(|style| style.bg(rgb(0x232429)).text_color(rgb(TEXT)))
                })
                .on_click(move |_, window, cx| on_select(index, window, cx))
                .child(item.label)
        })
        .collect::<Vec<_>>();

    div()
        .id(id)
        .flex_none()
        .h(TAB_BAR_HEIGHT)
        .w_full()
        .flex()
        .items_end()
        .gap_1()
        .px_2()
        .border_b_1()
        .border_color(rgb(BORDER))
        .bg(rgb(PANEL))
        .overflow_x_scroll()
        .children(tabs)
        .child(
            div()
                .id((id, u32::MAX))
                .flex_none()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .px_3()
                .text_lg()
                .text_color(rgb(MUTED))
                .cursor_pointer()
                .hover(|style| style.bg(rgb(0x232429)).text_color(rgb(TEXT)))
                .on_click(move |_, window, cx| on_new(window, cx))
                .child("+"),
        )
}
