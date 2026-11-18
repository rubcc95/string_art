//! Application state, screens and window chrome wiring.

mod chrome;
mod import;
mod settings;
mod sidebar;
mod tabs;
mod toast;
mod workspace;

use gpui::{AsyncApp, Context, Entity, Render, Subscription, Window, div, prelude::*, rgb};

use crate::import::ImportedImage;
use crate::ui;

use settings::{Settings, SettingsPanel};

/// One imported image and everything derived from it.
struct ImageTab {
    image: ImportedImage,
    /// The settings currently being edited for this image. Computations keep
    /// their own frozen copy so editing these does not alter them.
    settings: Settings,
    /// The editor that owns the numeric fields and sliders.
    panel: Entity<SettingsPanel>,
    /// Keeps the panel -> model subscription alive for as long as the tab.
    _settings_subscription: Subscription,
    computations: Vec<ComputationTab>,
    /// `None` shows the settings view. Computations are selected by index.
    active: Option<usize>,
}

/// One computation started from an image's settings.
struct ComputationTab {
    name: String,
    /// The settings the computation was started with.
    #[allow(dead_code)] // Used once computations actually run.
    settings: Settings,
}

pub struct AppView {
    images: Vec<ImageTab>,
    active_image: Option<usize>,
    /// Whether the compact image picker sheet is open.
    image_picker_open: bool,
    busy: bool,
    /// Whether a file picker was opened and has not reported back yet. Avoids
    /// stacking dialogs when the picker button is pressed several times before
    /// the first one shows up.
    picker_pending: bool,
    /// Messages floating above the application content, oldest first.
    toasts: Vec<ui::ToastEntry>,
    /// Identifies each toast so its timers only affect their own entry.
    next_toast_id: u64,
    /// Drives the drop zone highlight. Cleared as soon as the drag ends.
    drag_hover: Option<ui::DragHover>,
    /// Kept for a while after `drag_hover` is cleared so it can fade out.
    drag_overlay: Option<ui::DragOverlay>,
    drag_transition: u64,
    /// Set between pressing the title bar and actually moving the window, so a
    /// plain click (or a double click) does not start a window drag.
    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "macos"))]
    titlebar_should_move: bool,
    window_subscriptions: Vec<Subscription>,
}

/// Initialises the platform-specific import sources (file picker and drops).
pub fn init_import(view: Entity<AppView>, app: AsyncApp) {
    import::sources::init(view, app);
}

impl AppView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            images: Vec::new(),
            active_image: None,
            image_picker_open: false,
            busy: false,
            picker_pending: false,
            toasts: Vec::new(),
            next_toast_id: 0,
            drag_hover: None,
            drag_overlay: None,
            drag_transition: 0,
            #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "macos"))]
            titlebar_should_move: false,
            window_subscriptions: Vec::new(),
        }
    }

    /// Commits any in-progress numeric edit in the active image's panel.
    fn commit_active_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.active_image {
            if let Some(panel) = self.images.get(index).map(|tab| tab.panel.clone()) {
                panel.update(cx, |panel, cx| panel.commit(window, cx));
            }
        }
    }

    /// Opens the settings view of the active image without closing any
    /// computation tab.
    fn show_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_active_settings(window, cx);
        if let Some(tab) = self
            .active_image
            .and_then(|index| self.images.get_mut(index))
        {
            tab.active = None;
        }
        cx.notify();
    }

    fn select_image(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_active_settings(window, cx);
        if index < self.images.len() {
            self.active_image = Some(index);
            self.image_picker_open = false;
        }
        cx.notify();
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.active_image {
            Some(_) => self.workspace(window, cx).into_any_element(),
            None => self.import_screen(window, cx).into_any_element(),
        };
        let drag_overlay = self.drag_overlay.clone();
        let picker_open = self.image_picker_open;
        let toasts = self.toasts.clone();

        let root = div()
            .id("app")
            .relative()
            .size_full()
            .bg(rgb(ui::BG))
            .text_color(rgb(ui::TEXT))
            .child(content)
            .children(ui::title_bar::resize_handles(window))
            .when(picker_open, |root| root.child(self.image_picker(cx)))
            .when_some(drag_overlay, |root, overlay| {
                root.child(ui::drag_overlay(overlay))
            })
            .when(!toasts.is_empty(), |root| {
                root.child(ui::toast_layer(toasts.into_iter().map(|entry| {
                    let id = entry.id;
                    ui::toast_card(&entry)
                        .on_click(cx.listener(move |this, _, _, cx| this.dismiss_toast(id, cx)))
                        .into_any_element()
                })))
            });

        import::sources::attach_drop(root, cx)
    }
}
