//! One workspace: an imported image and everything derived from it.

mod ellipse;
mod settings;
mod tabs;

use gpui::{AnyElement, Context, Entity, EventEmitter, Render, Subscription, Window, div, prelude::*};

use crate::import::ImportedImage;
use crate::ui;

use super::settings::{Settings, SettingsPanel, SettingsPanelEvent};
use super::AppView;

pub(crate) use ellipse::EllipseState;

/// Something the global shell needs to know about a workspace.
#[derive(Clone, Copy, Debug)]
pub(crate) enum WorkspaceEvent {
    ComputationCreated,
}

pub(crate) struct Workspace {
    image: ImportedImage,
    settings: Settings,
    ellipse: EllipseState,
    panel: Entity<SettingsPanel>,
    _settings_subscription: Subscription,
    computations: Vec<ComputationTab>,
    /// `None` shows the settings view. Computations are selected by index.
    active: Option<usize>,
}

/// One computation started from an image's settings.
pub(crate) struct ComputationTab {
    name: String,
    /// The settings the computation was started with.
    #[allow(dead_code)] // Used once computations actually run.
    settings: Settings,
}

impl EventEmitter<WorkspaceEvent> for Workspace {}

impl Workspace {
    pub fn new(image: ImportedImage, cx: &mut Context<Self>) -> Self {
        let settings = Settings::default();
        let panel = cx.new(|cx| SettingsPanel::new(settings, cx));
        let subscription = cx.subscribe(&panel, |this, _, event, cx| {
            let SettingsPanelEvent::Changed(settings) = *event;
            this.settings = settings;
            cx.notify();
        });

        Self {
            image,
            settings,
            ellipse: EllipseState::default(),
            panel,
            _settings_subscription: subscription,
            computations: Vec::new(),
            active: None,
        }
    }

    pub fn image(&self) -> &ImportedImage {
        &self.image
    }

    pub fn computation_count(&self) -> usize {
        self.computations.len()
    }

    pub(super) fn commit_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.panel.update(cx, |panel, cx| panel.commit(window, cx));
    }

    pub(super) fn show_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_settings(window, cx);
        self.active = None;
        cx.notify();
    }

    fn content(
        &mut self,
        split: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match self.active.and_then(|active| self.computations.get(active)) {
            None => self.settings_view(split, window, cx).into_any_element(),
            Some(computation) => ui::empty_state(
                computation.name.clone(),
                "El resultado de la computación aparecerá aquí.",
            )
            .into_any_element(),
        }
    }

    fn split_layout(window: &Window) -> bool {
        let width = window.viewport_size().width;
        let mobile = width < ui::MOBILE_MAX_WIDTH;
        let rail = !mobile && width < ui::FULL_SIDEBAR_MIN_WIDTH;
        let sidebar_width = if rail {
            ui::SIDEBAR_RAIL_WIDTH
        } else {
            ui::SIDEBAR_WIDTH
        };
        !mobile && width - sidebar_width >= ui::SETTINGS_SPLIT_MIN_WIDTH
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let split = Self::split_layout(window);
        let mut main = div()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col();

        if !self.computations.is_empty() {
            main = main.child(self.computation_tabs(cx));
        }

        main.child(self.content(split, window, cx))
    }
}

impl AppView {
    pub(super) fn workspace(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(index) = self.active_workspace else {
            return div().into_any_element();
        };
        let Some(workspace) = self.workspaces.get(index).cloned() else {
            return div().into_any_element();
        };

        let width = window.viewport_size().width;
        let mobile = width < ui::MOBILE_MAX_WIDTH;
        let rail = !mobile && width < ui::FULL_SIDEBAR_MIN_WIDTH;

        let trailing = if mobile {
            self.image_switcher(&workspace, cx).into_any_element()
        } else {
            div().into_any_element()
        };

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
                    .child(workspace),
            )
            .into_any_element()
    }
}
