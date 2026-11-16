//! Application state, screens and window chrome wiring.

mod chrome;
mod import;
mod workspace;

use gpui::{AsyncApp, Context, Entity, Render, Subscription, Window, div, prelude::*, rgb};

use crate::import::ImportedImage;
use crate::ui;

/// One imported image. Later it will also hold the computation settings and
/// its output; the application is already built around more than one tab.
struct Tab {
    image: ImportedImage,
}

/// What the application is showing about the file being dragged over it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DragHover {
    /// The dropped file can be imported. The name is only known on desktop.
    Accepted { name: Option<String> },
    /// The dropped file cannot be imported.
    Rejected { name: String },
    /// An import is running and drops are ignored for now.
    Busy,
}

/// The drag overlay, which outlives the drag itself while it fades out.
#[derive(Clone, Debug)]
pub struct DragOverlay {
    pub hover: DragHover,
    /// Whether the overlay is fading out after the drag ended.
    pub leaving: bool,
    /// Identifies each entry/exit so its animations start from the beginning
    /// and stale exit timers can be ignored.
    pub transition: u64,
}

pub struct AppView {
    tabs: Vec<Tab>,
    active: Option<usize>,
    busy: bool,
    error: Option<String>,
    /// Drives the drop zone highlight. Cleared as soon as the drag ends.
    drag_hover: Option<DragHover>,
    /// Kept for a while after `drag_hover` is cleared so it can fade out.
    drag_overlay: Option<DragOverlay>,
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
            tabs: Vec::new(),
            active: None,
            busy: false,
            error: None,
            drag_hover: None,
            drag_overlay: None,
            drag_transition: 0,
            #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "macos"))]
            titlebar_should_move: false,
            window_subscriptions: Vec::new(),
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.active.and_then(|index| self.tabs.get(index));
        let content = match active {
            Some(tab) => self.workspace(tab, window, cx).into_any_element(),
            None => self.import_screen(window, cx).into_any_element(),
        };
        let drag_overlay = self.drag_overlay.clone();

        let root = div()
            .id("app")
            .relative()
            .size_full()
            .bg(rgb(ui::BG))
            .text_color(rgb(ui::TEXT))
            .child(content)
            .children(ui::title_bar::resize_handles(window))
            .when_some(drag_overlay, |root, overlay| {
                root.child(ui::drag_overlay(overlay))
            });

        import::sources::attach_drop(root, cx)
    }
}
