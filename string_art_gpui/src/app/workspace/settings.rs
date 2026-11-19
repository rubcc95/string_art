//! Settings screen of one workspace: preview, form and start button.

use gpui::{
    AnyElement, Context, KeyDownEvent, ObjectFit, Window, div, img, prelude::*, px, rgb, size,
};
use rust_i18n::t;

use crate::i18n;
use crate::ui;

use super::Workspace;

impl Workspace {
    pub(super) fn settings_view(
        &mut self,
        split: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.start_focus_subscriptions.is_empty() {
            let start_focus = self.panel.read(cx).start_focus();
            self.start_focus_subscriptions.push(cx.on_focus(
                &start_focus,
                window,
                |_, window, cx| {
                    cx.defer_in(window, |_, _, cx| cx.notify());
                },
            ));
            self.start_focus_subscriptions.push(cx.on_blur(
                &start_focus,
                window,
                |_, window, cx| {
                    cx.defer_in(window, |_, _, cx| cx.notify());
                },
            ));
        }

        let preview_size = self.image.preview.size(0);
        let image_size = size(px(self.image.width as f32), px(self.image.height as f32));
        let on_pointer = cx.listener(|workspace, event: &ui::EllipsePointerEvent, _window, cx| {
            if workspace.ellipse.pointer(event) {
                workspace.sync_ellipse_aspect(cx);
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

    /// Copies the aspect ratio of the resized ellipse to the physical size of
    /// the work, so the drawing is never stretched. Moving the ellipse does
    /// not change the ratio and therefore does nothing here.
    fn sync_ellipse_aspect(&mut self, cx: &mut Context<Self>) {
        let frame = self.ellipse.frame;
        if frame.width <= 0.
            || frame.height <= 0.
            || self.image.width == 0
            || self.image.height == 0
        {
            return;
        }

        let aspect = (f64::from(frame.height) * f64::from(self.image.height))
            / (f64::from(frame.width) * f64::from(self.image.width));
        if (f64::from(self.settings.aspect()) - aspect).abs() <= 1e-5 {
            return;
        }

        self.panel
            .update(cx, |panel, cx| panel.set_ellipse_aspect(aspect, cx));
    }

    fn settings_form(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let name = self.image.name.clone();
        let dimensions = t!(
            "settings.dimensions",
            width = i18n::format_display_number(f64::from(self.image.width)),
            height = i18n::format_display_number(f64::from(self.image.height))
        )
        .to_string();
        let panel = self.panel.clone();
        let start_focus = self.panel.read(cx).start_focus();

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
                            .child(t!("settings.title").to_string()),
                    )
                    .child(div().text_sm().text_color(rgb(ui::TEXT)).child(name))
                    .child(div().text_xs().text_color(rgb(ui::MUTED)).child(dimensions)),
            )
            .child(panel)
            .child(
                div().flex().flex_col().gap_2().child(
                    ui::primary_button("start-computation", t!("settings.start").to_string(), true)
                        .w_full()
                        .track_focus(&start_focus)
                        .focus(|style| style.bg(rgb(ui::ACCENT_HOVER)))
                        .on_key_down(cx.listener(|workspace, event: &KeyDownEvent, window, cx| {
                            match event.keystroke.key.as_str() {
                                "enter" => {
                                    workspace.start_computation(window, cx);
                                    cx.stop_propagation();
                                }
                                "tab" => {
                                    if event.keystroke.modifiers.shift {
                                        window.focus_prev(cx);
                                    } else {
                                        window.focus_next(cx);
                                    }
                                    cx.stop_propagation();
                                }
                                _ => {}
                            }
                        }))
                        .on_click(cx.listener(|workspace, _, window, cx| {
                            workspace.start_computation(window, cx);
                        })),
                ),
            )
    }

    pub(super) fn start_computation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_settings(window, cx);
        let settings = self.panel.read(cx).settings();
        let number = self.computations.len() + 1;
        self.computations
            .push(super::ComputationTab { number, settings });
        self.active = Some(number - 1);
        cx.emit(super::WorkspaceEvent::ComputationCreated);
        cx.notify();
    }
}
