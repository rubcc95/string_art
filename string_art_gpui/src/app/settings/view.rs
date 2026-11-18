//! Settings screen composition: preview, header, panel and start button.

use gpui::{div, img, prelude::*, px, rgb, size, AnyElement, Context, ObjectFit, Window};

use crate::ui;

use crate::app::{AppView, ComputationTab, ImageTab};

impl AppView {
    /// The settings view. It has no tab of its own: it is the landing view of
    /// an image and is shown whenever no computation tab is active.
    pub(in crate::app) fn settings_view(
        &self,
        index: usize,
        tab: &ImageTab,
        split: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let preview_size = tab.image.preview.size(0);
        let preview = ui::preview_card(
            img(tab.image.preview.clone())
                .size_full()
                .object_fit(ObjectFit::Contain),
            ui::ellipse_overlay(
                preview_size,
                size(px(tab.image.width as f32), px(tab.image.height as f32)),
                ui::EllipseFrame::FULL,
                tab.settings.nail_count,
                tab.settings.nail_radius_px,
            ),
            !split,
        );
        let form = self.settings_form(index, tab, cx);

        if split {
            div()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .flex()
                .gap_6()
                .p_6()
                .child(preview)
                .child(
                    div()
                        .id("settings-form")
                        .flex_none()
                        .w(px(360.))
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(form),
                )
                .into_any_element()
        } else {
            div()
                .id("settings-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(
                    div()
                        .w_full()
                        .max_w(px(560.))
                        .mx_auto()
                        .flex()
                        .flex_col()
                        .gap_5()
                        .p_6()
                        .child(preview)
                        .child(form),
                )
                .into_any_element()
        }
    }

    fn settings_form(
        &self,
        index: usize,
        tab: &ImageTab,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let name = tab.image.name.clone();
        let dimensions = format!("{} × {} px", tab.image.width, tab.image.height);
        let panel = tab.panel.clone();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(ui::TEXT))
                            .child("Configuration"),
                    )
                    .child(div().text_sm().text_color(rgb(ui::TEXT)).child(name))
                    .child(div().text_xs().text_color(rgb(ui::MUTED)).child(dimensions)),
            )
            .child(panel)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        ui::primary_button("start-computation", "Start", true)
                            .w_full()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.start_computation(index, window, cx);
                            })),
                    ),
            )
    }

    /// Starts a computation from the current settings and selects its tab.
    pub(in crate::app) fn start_computation(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(panel) = self.images.get(index).map(|tab| tab.panel.clone()) {
            panel.update(cx, |panel, cx| panel.commit(window, cx));
        }

        let settings = self
            .images
            .get(index)
            .map(|tab| tab.panel.read(cx).settings())
            .unwrap_or_default();

        if let Some(tab) = self.images.get_mut(index) {
            let number = tab.computations.len() + 1;
            tab.computations.push(ComputationTab {
                name: format!("Computation {number}"),
                settings,
            });
            tab.active = Some(number - 1);
        }
        cx.notify();
    }
}
