//! Shared colours and reusable UI components.

use std::time::Duration;

use gpui::{Pixels, px};

mod bottom_sheet;
mod button;
mod drag_overlay;
mod empty_state;
mod number_input;
mod preview;
mod sidebar;
mod slider_field;
mod switcher;
mod tab_bar;
mod toast;

pub mod title_bar;

pub use bottom_sheet::bottom_sheet;
pub use button::{icon_button, primary_button, secondary_button};
pub use drag_overlay::{DragHover, DragOverlay, drag_overlay};
pub use empty_state::empty_state;
pub use preview::preview_card;
pub use sidebar::{ListItem, list_item, sidebar, sidebar_import_button};
pub use slider_field::{SliderField, SliderFieldEvent};
pub use switcher::switcher_button;
pub use tab_bar::{TabItem, tab_bar};
#[allow(unused_imports)] // Icons and styles are the public API for custom toasts.
pub use toast::{
    TOAST_DURATION, TOAST_FADE, Toast, ToastEntry, ToastIcon, ToastStyle, toast_card, toast_layer,
};

/// Duration of both the entry and the exit of the drag overlay.
pub const DRAG_OVERLAY_FADE: Duration = Duration::from_secs(1);

pub const BG: u32 = 0x141417;
pub const PANEL: u32 = 0x1b1b20;
pub const PANEL_DRAGGING: u32 = 0x172033;
pub const BORDER: u32 = 0x2e2f36;
pub const TEXT: u32 = 0xe7e7ea;
pub const MUTED: u32 = 0x9b9ba4;
pub const FAINT: u32 = 0x6c6c75;
pub const ACCENT: u32 = 0x2563eb;
pub const ACCENT_HOVER: u32 = 0x1d4ed8;
pub const DISABLED: u32 = 0x2a2b31;
pub const DISABLED_TEXT: u32 = 0x74747d;
pub const ERROR_BG: u32 = 0x2a1517;
pub const ERROR_BORDER: u32 = 0x7f2d34;
pub const ERROR_TEXT: u32 = 0xf4a3a7;
pub const PREVIEW_BG: u32 = 0x0f0f12;

/// Below this width the image sidebar is replaced by the title-bar picker.
pub const MOBILE_MAX_WIDTH: Pixels = px(720.);
/// Width from which the full image sidebar is shown instead of the rail.
pub const FULL_SIDEBAR_MIN_WIDTH: Pixels = px(1000.);
pub const SIDEBAR_WIDTH: Pixels = px(224.);
pub const SIDEBAR_RAIL_WIDTH: Pixels = px(64.);
/// Minimum content width (after the sidebar) for preview and form to sit side
/// by side.
pub const SETTINGS_SPLIT_MIN_WIDTH: Pixels = px(820.);
pub const TAB_BAR_HEIGHT: Pixels = px(38.);
