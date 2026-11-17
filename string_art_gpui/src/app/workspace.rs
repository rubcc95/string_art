//! Workspace layout for one imported image.

use gpui::{AnyElement, Context, Window, div, prelude::*};

use crate::ui;

use super::{AppView, ImageTab};

impl AppView {
    pub(super) fn workspace(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(index) = self.active_image else {
            return div().into_any_element();
        };
        let Some(tab) = self.images.get(index) else {
            return div().into_any_element();
        };

        let width = window.viewport_size().width;
        let mobile = width < ui::MOBILE_MAX_WIDTH;
        let rail = !mobile && width < ui::FULL_SIDEBAR_MIN_WIDTH;
        let sidebar_width = if rail {
            ui::SIDEBAR_RAIL_WIDTH
        } else {
            ui::SIDEBAR_WIDTH
        };
        let content_width = width - sidebar_width;
        let split = !mobile && content_width >= ui::SETTINGS_SPLIT_MIN_WIDTH;

        let trailing = if mobile {
            self.image_switcher(tab, cx).into_any_element()
        } else {
            div().into_any_element()
        };

        let main = div()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .when(!tab.computations.is_empty(), |main| {
                main.child(self.computation_tabs(index, tab, cx))
            })
            .child(self.workspace_content(index, tab, split, window, cx));

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
                    .when(!mobile, |body| {
                        body.child(self.image_sidebar(index, rail, window, cx))
                    })
                    .child(main),
            )
            .into_any_element()
    }

    fn workspace_content(
        &self,
        index: usize,
        tab: &ImageTab,
        split: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match tab.active.and_then(|active| tab.computations.get(active)) {
            None => self
                .settings_view(index, tab, split, window, cx)
                .into_any_element(),
            Some(computation) => ui::empty_state(
                computation.name.clone(),
                "El resultado de la computación aparecerá aquí.",
            )
            .into_any_element(),
        }
    }
}
