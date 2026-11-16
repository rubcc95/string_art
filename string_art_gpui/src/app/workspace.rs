//! Workspace screen for an imported image.

use gpui::{Context, ObjectFit, Window, div, img, prelude::*, rgb};

use crate::ui;

use super::{AppView, Tab};

impl AppView {
    pub(super) fn workspace(
        &self,
        tab: &Tab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let busy = self.busy;
        let error = self.error.clone();
        let name = tab.image.name.clone();
        let dimensions = format!("{} × {} px", tab.image.width, tab.image.height);
        let preview = tab.image.preview.clone();

        let trailing = div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(ui::FAINT))
                    .child("Settings come next"),
            )
            .child(ui::primary_button("continue", "Continue", false));

        div()
            .size_full()
            .flex()
            .flex_col()
            .child(self.title_bar(trailing, window, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_6()
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .w_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .p_4()
                            .rounded_xl()
                            .border_1()
                            .border_color(rgb(ui::BORDER))
                            .bg(rgb(0x0f0f12))
                            .child(img(preview).size_full().object_fit(ObjectFit::Contain)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().text_sm().text_color(rgb(ui::TEXT)).child(name))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(ui::MUTED))
                                            .child(dimensions),
                                    ),
                            )
                            .when(busy, |row| {
                                row.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(ui::MUTED))
                                        .child("Importing…"),
                                )
                            }),
                    )
                    .when(error.is_some(), |column| {
                        column.child(ui::error_banner(error.as_deref().unwrap_or_default()))
                    }),
            )
    }
}
