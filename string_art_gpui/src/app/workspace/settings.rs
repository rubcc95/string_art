//! Settings screen of one workspace: preview, form and start button.

use gpui::{AnyElement, Context, ObjectFit, Window, div, img, prelude::*, px, rgb, size};

use crate::ui;

use super::Workspace;

impl Workspace {
    pub(super) fn settings_view(
        &mut self,
        split: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let preview_size = self.image.preview.size(0);
        let image_size = size(px(self.image.width as f32), px(self.image.height as f32));
        let on_pointer = cx.listener(|workspace, event: &ui::EllipsePointerEvent, _window, cx| {
            if workspace.ellipse.pointer(event) {
                cx.notify();
            }
        });
        let preview = ui::preview_card(
            img(self.image.preview.clone())
                .size_full()
                .object_fit(ObjectFit::Contain),
            ui::ellipse_overlay(
                preview_size,
                image_size,
                self.ellipse.frame,
                self.ellipse.hover,
                self.ellipse.arrows_seen,
                self.settings.nail_count,
                self.settings.nail_radius_px,
                on_pointer,
            ),
            !split,
        );
        let form = self.settings_form(cx);

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

    fn settings_form(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let name = self.image.name.clone();
        let dimensions = format!("{} × {} px", self.image.width, self.image.height);
        let panel = self.panel.clone();

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
                            .on_click(
                                cx.listener(|workspace, _, window, cx| {
                                    workspace.start_computation(window, cx);
                                }),
                            ),
                    ),
            )
    }

    pub(super) fn start_computation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.commit_settings(window, cx);
        let settings = self.panel.read(cx).settings();
        let number = self.computations.len() + 1;
        self.computations.push(super::ComputationTab {
            name: format!("Computation {number}"),
            settings,
        });
        self.active = Some(number - 1);
        cx.emit(super::WorkspaceEvent::ComputationCreated);
        cx.notify();
    }
}
