//! Image preview frame shared by the import and settings screens.

use gpui::{Div, IntoElement, div, prelude::*, px, rgb};

use super::{BORDER, PREVIEW_BG};

/// Wraps an image in the standard preview frame. In compact mode it has a
/// fixed height; otherwise it takes the space the parent gives it.
pub fn preview_card(content: impl IntoElement, compact: bool) -> Div {
    let card = div()
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h_full()
        .p_3()
        .rounded_xl()
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgb(PREVIEW_BG))
        .child(content);

    if compact {
        div().flex_none().w_full().h(px(200.)).child(card)
    } else {
        div().flex_1().min_h_0().min_w_0().w_full().child(card)
    }
}
