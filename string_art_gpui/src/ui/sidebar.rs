//! Sidebar shell and its rows.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, IntoElement, Pixels, SharedString, Stateful, Window, div, prelude::*, rgb,
};
use rust_i18n::t;

use super::{BORDER, MUTED, PANEL, TEXT};

/// A row in a sidebar or picker list.
pub struct ListItem {
    pub leading: AnyElement,
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    pub trailing: Option<AnyElement>,
    pub active: bool,
    pub height: Pixels,
}

/// One list row. In compact mode only the leading element is shown, which is
/// what the icon-only sidebar rail uses.
pub fn list_item(id: (&'static str, usize), item: ListItem, compact: bool) -> Stateful<Div> {
    let ListItem {
        leading,
        title,
        subtitle,
        trailing,
        active,
        height,
    } = item;

    let row = div()
        .id(id)
        .w_full()
        .h(height)
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .rounded_lg()
        .cursor_pointer()
        .when(active, |row| row.bg(gpui::rgba(0x2563eb33)))
        .when(!active, |row| row.hover(|style| style.bg(rgb(0x232429))))
        .child(leading);

    if compact {
        row.justify_center()
    } else {
        let title_color = if active { TEXT } else { MUTED };
        let text = if let Some(subtitle) = subtitle {
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_sm()
                        .text_color(rgb(title_color))
                        .child(title),
                )
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(subtitle),
                )
        } else {
            div().flex_1().min_w_0().flex().flex_col().child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(rgb(title_color))
                    .child(title),
            )
        };

        row.justify_start()
            .child(text)
            .when_some(trailing, |row, trailing| row.child(trailing))
    }
}

/// The sidebar shell: width, border, scrollable list and an optional footer.
pub fn sidebar(
    id: &'static str,
    width: Pixels,
    compact: bool,
    items: Vec<ListItem>,
    footer: Option<AnyElement>,
    on_select: impl Fn(usize, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let on_select = Rc::new(on_select);
    let rows = items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let on_select = on_select.clone();
            list_item((id, index), item, compact).on_click(move |_, window, cx| {
                on_select(index, window, cx);
            })
        })
        .collect::<Vec<_>>();

    div()
        .id(id)
        .flex_none()
        .w(width)
        .h_full()
        .flex()
        .flex_col()
        .gap_2()
        .p_2()
        .border_r_1()
        .border_color(rgb(BORDER))
        .bg(rgb(PANEL))
        .child(
            div()
                .id((id, u32::MAX))
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap_1()
                .children(rows),
        )
        .when_some(footer, |sidebar, footer| sidebar.child(footer))
}

/// A compact import button for the sidebar footer. The caller attaches its
/// click handler.
pub fn sidebar_import_button(compact: bool, enabled: bool) -> Stateful<Div> {
    if compact {
        super::icon_button("sidebar-import", "+", enabled).mx_auto()
    } else {
        super::secondary_button(
            "sidebar-import",
            format!("+  {}", t!("sidebar.import")),
            enabled,
        )
        .w_full()
    }
}
