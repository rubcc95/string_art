//! Image sidebar, compact switcher and mobile picker.

use gpui::{Context, Entity, ObjectFit, Window, div, img, prelude::*, px, rgb};

use crate::ui;

use super::AppView;
use super::workspace::Workspace;

impl AppView {
    pub(super) fn image_sidebar(
        &self,
        active: usize,
        rail: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let items = self
            .workspaces
            .iter()
            .enumerate()
            .map(|(index, workspace)| {
                let workspace = workspace.read(cx);
                let image = workspace.image();
                ui::ListItem {
                    leading: img(image.preview.clone())
                        .size(px(32.))
                        .rounded_md()
                        .object_fit(ObjectFit::Cover)
                        .into_any_element(),
                    title: image.name.clone().into(),
                    subtitle: None,
                    trailing: Some(
                        div()
                            .text_xs()
                            .text_color(rgb(ui::FAINT))
                            .child(workspace.computation_count().to_string())
                            .into_any_element(),
                    ),
                    active: index == active,
                    height: px(44.),
                }
            })
            .collect::<Vec<_>>();

        let footer = ui::sidebar_import_button(rail, !self.busy && !self.picker_pending)
            .on_click(cx.listener(|this, _, window, cx| this.pick_image(window, cx)))
            .into_any_element();

        let entity = cx.entity();
        ui::sidebar(
            "image-sidebar",
            if rail {
                ui::SIDEBAR_RAIL_WIDTH
            } else {
                ui::SIDEBAR_WIDTH
            },
            rail,
            items,
            Some(footer),
            move |index, window, cx| {
                entity.update(cx, |this, cx| this.select_image(index, window, cx));
            },
        )
    }

    pub(super) fn image_switcher(
        &self,
        workspace: &Entity<Workspace>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (preview, name) = {
            let workspace = workspace.read(cx);
            let image = workspace.image();
            (image.preview.clone(), image.name.clone())
        };

        ui::switcher_button(
            "image-switcher",
            img(preview)
                .size(px(22.))
                .rounded_sm()
                .object_fit(ObjectFit::Cover)
                .into_any_element(),
            name,
            cx.listener(|this, _, window, cx| {
                this.commit_active_settings(window, cx);
                this.image_picker_open = !this.image_picker_open;
                cx.notify();
            }),
        )
    }

    pub(super) fn image_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.active_workspace;
        let rows = self
            .workspaces
            .iter()
            .enumerate()
            .map(|(index, workspace)| {
                let workspace = workspace.read(cx);
                let image = workspace.image();
                let item = ui::ListItem {
                    leading: img(image.preview.clone())
                        .size(px(38.))
                        .rounded_md()
                        .object_fit(ObjectFit::Cover)
                        .into_any_element(),
                    title: image.name.clone().into(),
                    subtitle: Some(
                        format!(
                            "{} × {} px · {} computaciones",
                            image.width,
                            image.height,
                            workspace.computation_count()
                        )
                        .into(),
                    ),
                    trailing: None,
                    active: Some(index) == active,
                    height: px(52.),
                };
                ui::list_item(("picker-row", index), item, false).on_click(
                    cx.listener(move |this, _, window, cx| this.select_image(index, window, cx)),
                )
            })
            .collect::<Vec<_>>();

        let content = div()
            .id("image-picker-list")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_1()
            .p_3()
            .children(rows)
            .into_any_element();

        let footer = div()
            .flex_none()
            .p_3()
            .pt_0()
            .child(
                ui::primary_button(
                    "picker-import",
                    "Importar imagen",
                    !self.busy && !self.picker_pending,
                )
                .w_full()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.image_picker_open = false;
                    this.pick_image(window, cx);
                })),
            )
            .into_any_element();

        let entity = cx.entity();
        ui::bottom_sheet(
            "image-picker",
            "Imágenes",
            content,
            Some(footer),
            move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.image_picker_open = false;
                    cx.notify();
                });
            },
        )
    }
}
