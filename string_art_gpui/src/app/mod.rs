//! Application state, screens and window chrome wiring.

mod chrome;
mod import;
mod settings;
mod sidebar;
mod toast;
mod workspace;

use gpui::{AsyncApp, Context, Entity, Render, Subscription, Window, div, prelude::*, rgb};

use crate::ui;

use workspace::Workspace;

pub struct AppView {
    workspaces: Vec<Entity<Workspace>>,
    active_workspace: Option<usize>,
    /// Keeps the workspace -> shell subscriptions alive.
    _workspace_subscriptions: Vec<Subscription>,
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
            workspaces: Vec::new(),
            active_workspace: None,
            _workspace_subscriptions: Vec::new(),
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

    /// Commits any in-progress numeric edit in the active workspace.
    fn commit_active_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(workspace) = self
            .active_workspace
            .and_then(|index| self.workspaces.get(index))
            .cloned()
        else {
            return;
        };
        workspace.update(cx, |workspace, cx| workspace.commit_settings(window, cx));
    }

    fn select_image(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_active_settings(window, cx);
        if index < self.workspaces.len() {
            self.active_workspace = Some(index);
            self.image_picker_open = false;
        }
        cx.notify();
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.active_workspace {
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
            .font_family(".SystemUIFont")
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
