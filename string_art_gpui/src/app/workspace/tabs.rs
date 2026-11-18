//! Computation tabs of one workspace.

use gpui::{Context, prelude::*};

use crate::ui;

use super::Workspace;

impl Workspace {
    pub(super) fn computation_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self
            .computations
            .iter()
            .enumerate()
            .map(|(index, computation)| ui::TabItem {
                label: computation.name.clone().into(),
                active: self.active == Some(index),
            })
            .collect::<Vec<_>>();

        let entity = cx.entity();
        let new_entity = entity.clone();
        ui::tab_bar(
            "computation-tabs",
            items,
            move |index, window, cx| {
                entity.update(cx, |workspace, cx| {
                    workspace.commit_settings(window, cx);
                    workspace.active = Some(index);
                    cx.notify();
                });
            },
            move |window, cx| {
                new_entity.update(cx, |workspace, cx| workspace.show_settings(window, cx));
            },
        )
    }
}
