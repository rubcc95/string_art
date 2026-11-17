//! Import feature state: choosing a file, tracking busy/error and spawning
//! decodes.

mod screen;
pub(super) mod sources;

use gpui::{AppContext as _, Context, Window};

use crate::import::{ImportError, ImportedImage, decode};
use crate::ui;
use crate::ui::{DragHover, DragOverlay, Toast};

use super::settings::{Settings, SettingsPanel, SettingsPanelEvent};
use super::{AppView, ImageTab};

impl AppView {
    fn begin_import(&mut self, cx: &mut Context<Self>) -> bool {
        if self.busy {
            return false;
        }
        self.busy = true;
        cx.notify();
        true
    }

    fn finish_import(
        &mut self,
        result: Result<ImportedImage, ImportError>,
        cx: &mut Context<Self>,
    ) {
        self.busy = false;
        match result {
            Ok(image) => {
                let settings = Settings::default();
                let panel = cx.new(|cx| SettingsPanel::new(settings, cx));
                let subscription = cx.subscribe(&panel, |this, source, event, cx| {
                    let SettingsPanelEvent::Changed(settings) = *event;
                    if let Some(tab) = this.images.iter_mut().find(|tab| tab.panel == source) {
                        tab.settings = settings;
                        cx.notify();
                    }
                });
                self.images.push(ImageTab {
                    image,
                    settings,
                    panel,
                    _settings_subscription: subscription,
                    computations: Vec::new(),
                    active: None,
                });
                self.active_image = Some(self.images.len() - 1);
            }
            Err(error) => {
                self.toast(Toast::error("Import failed", Some(error.to_string())), cx);
            }
        }
        cx.notify();
    }

    pub fn import_bytes(&mut self, name: String, bytes: Vec<u8>, cx: &mut Context<Self>) {
        if !self.begin_import(cx) {
            return;
        }

        let background = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            let result = background.spawn(async move { decode(name, &bytes) }).await;
            let _ = this.update(cx, |this, cx| this.finish_import(result, cx));
        })
        .detach();
    }

    #[allow(dead_code)] // Only the native drop path asks.
    pub fn is_busy(&self) -> bool {
        self.busy
    }

    /// Updates the drag feedback shown while a file is over the window.
    ///
    /// When the drag ends the overlay is not removed immediately: it is kept
    /// around in a "leaving" state long enough to fade out.
    pub fn set_drag_hover(&mut self, hover: Option<DragHover>, cx: &mut Context<Self>) {
        let Some(hover) = hover else {
            let had_hover = self.drag_hover.take().is_some();
            if !self
                .drag_overlay
                .as_ref()
                .is_some_and(|overlay| !overlay.leaving)
            {
                if had_hover {
                    cx.notify();
                }
                return;
            }

            self.drag_transition += 1;
            let transition = self.drag_transition;
            if let Some(overlay) = &mut self.drag_overlay {
                overlay.leaving = true;
                overlay.transition = transition;
            }

            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(ui::DRAG_OVERLAY_FADE).await;
                let _ = this.update(cx, |this, cx| {
                    let current = this.drag_overlay.as_ref().map(|overlay| overlay.transition);
                    if current == Some(transition) {
                        this.drag_overlay = None;
                        cx.notify();
                    }
                });
            })
            .detach();

            cx.notify();
            return;
        };

        let already_entering = self
            .drag_overlay
            .as_ref()
            .is_some_and(|overlay| !overlay.leaving);
        let changed = self.drag_hover.as_ref() != Some(&hover) || !already_entering;

        self.drag_hover = Some(hover.clone());
        if already_entering {
            if let Some(overlay) = &mut self.drag_overlay {
                overlay.hover = hover;
            }
        } else {
            self.drag_transition += 1;
            self.drag_overlay = Some(DragOverlay {
                hover,
                leaving: false,
                transition: self.drag_transition,
            });
        }

        if changed {
            cx.notify();
        }
    }

    pub fn pick_image(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        sources::pick(window, cx);
    }

    /// Reports an import failure without leaving the import UI in a busy state.
    pub fn import_error(&mut self, message: String, cx: &mut Context<Self>) {
        self.busy = false;
        self.toast(Toast::error("Import failed", Some(message)), cx);
    }

    /// Drop feedback for platforms that only report whether files are over the
    /// window (the browser hides their names until they are dropped).
    #[allow(dead_code)] // Only the web drop listener uses it.
    pub fn set_dragging(&mut self, dragging: bool, cx: &mut Context<Self>) {
        let hover = dragging.then_some(if self.busy {
            DragHover::Busy
        } else {
            DragHover::Accepted { name: None }
        });
        self.set_drag_hover(hover, cx);
    }
}
