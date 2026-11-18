//! Image preview frame shared by the import and settings screens.

use gpui::{Div, IntoElement, Pixels, div, prelude::*, px, rgb};

use super::{BORDER, PREVIEW_BG};

/// Padding between the panel border and its content.
const PADDING: Pixels = px(12.);

/// Wraps an image in the standard preview frame. In compact mode it has a
/// fixed height; otherwise it takes the space the parent gives it.
///
/// The overlay is positioned over the content area without taking part in the
/// layout, so it cannot change the size of the image.
pub fn preview_card(content: impl IntoElement, overlay: impl IntoElement, compact: bool) -> Div {
    let card = div()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h_full()
        .p(PADDING)
        .rounded_xl()
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgb(PREVIEW_BG))
        .child(content)
        .child(
            div()
                .absolute()
                .top(PADDING)
                .left(PADDING)
                .right(PADDING)
                .bottom(PADDING)
                .child(overlay),
        );

    if compact {
        div().flex_none().w_full().h(px(200.)).child(card)
    } else {
        div().flex_1().min_h_0().min_w_0().w_full().child(card)
    }
}
