//! Computation tabs for the active image.

use gpui::{Context, prelude::*};

use crate::ui;

use super::{AppView, ImageTab};

impl AppView {
    pub(super) fn computation_tabs(
        &self,
        image_index: usize,
        tab: &ImageTab,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let items = tab
            .computations
            .iter()
            .enumerate()
            .map(|(index, computation)| ui::TabItem {
                label: computation.name.clone().into(),
                active: tab.active == Some(index),
            })
            .collect::<Vec<_>>();

        let entity = cx.entity();
        let new_entity = entity.clone();
        ui::tab_bar(
            "computation-tabs",
            items,
            move |index, window, cx| {
                entity.update(cx, |this, cx| {
                    this.commit_active_settings(window, cx);
                    if let Some(tab) = this.images.get_mut(image_index) {
                        tab.active = Some(index);
                    }
                    cx.notify();
                });
            },
            move |window, cx| {
                new_entity.update(cx, |this, cx| this.show_settings(window, cx));
            },
        )
    }
}
