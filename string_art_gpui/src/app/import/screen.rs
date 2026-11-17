//! Import screen.

use gpui::{Context, Window, div, prelude::*, px, rgb};

use crate::app::AppView;
use crate::ui;
use crate::ui::DragHover;

impl AppView {
    pub(in crate::app) fn import_screen(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let busy = self.busy;
        let drag_accepted = matches!(self.drag_hover, Some(DragHover::Accepted { .. }));
        let drag_rejected = matches!(self.drag_hover, Some(DragHover::Rejected { .. }));

        let button_label = if busy {
            "Importing…"
        } else {
            "Choose an image"
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.title_bar(div(), window, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p_8()
                    .child(
                        div()
                            .w_full()
                            .max_w(px(760.))
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .id("drop-zone")
                                    .w_full()
                                    .h(px(420.))
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap_5()
                                    .rounded_xl()
                                    .border_1()
                                    .border_dashed()
                                    .border_color(rgb(if drag_rejected {
                                        ui::ERROR_BORDER
                                    } else if drag_accepted {
                                        ui::ACCENT
                                    } else {
                                        ui::BORDER
                                    }))
                                    .bg(rgb(if drag_rejected {
                                        ui::ERROR_BG
                                    } else if drag_accepted {
                                        ui::PANEL_DRAGGING
                                    } else {
                                        ui::PANEL
                                    }))
                                    .child(
                                        div()
                                            .text_2xl()
                                            .text_color(rgb(ui::TEXT))
                                            .child("Drop an image here"),
                                    )
                                    .child(div().text_sm().text_color(rgb(ui::MUTED)).child("or"))
                                    .child(
                                        ui::primary_button("import-image", button_label, !busy)
                                            .when(!busy, |button| {
                                                button.on_click(cx.listener(
                                                    |this, _, window, cx| {
                                                        this.pick_image(window, cx)
                                                    },
                                                ))
                                            }),
                                    )
                                    .child(
                                        div()
                                            .max_w(px(460.))
                                            .text_center()
                                            .text_xs()
                                            .text_color(rgb(ui::FAINT))
                                            .child(
                                                "PNG, JPEG, WebP, GIF, BMP, TIFF, ICO, PNM, QOI \
                                                 and TGA",
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}
