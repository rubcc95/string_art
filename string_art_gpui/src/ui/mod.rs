//! Shared colours and small pieces of the application chrome.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, Div, Interpolate as _, IntoElement, Rgba, Stateful, div,
    ease_out_quint, prelude::*, px, rgb, rgba,
};

use crate::app::{DragHover, DragOverlay};

pub mod title_bar;

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

pub fn primary_button(id: &'static str, label: &str, enabled: bool) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .px_6()
        .py_2()
        .rounded_lg()
        .bg(rgb(if enabled { ACCENT } else { DISABLED }))
        .text_sm()
        .text_color(rgb(if enabled { 0xffffff } else { DISABLED_TEXT }))
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(|style| style.bg(rgb(ACCENT_HOVER)))
        })
        .child(label.to_string())
}

pub fn error_banner(message: &str) -> Div {
    div()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(rgb(ERROR_BORDER))
        .bg(rgb(ERROR_BG))
        .px_4()
        .py_3()
        .text_sm()
        .text_color(rgb(ERROR_TEXT))
        .child(message.to_string())
}

/// Full-window hint shown while a file is being dragged over the application.
///
/// It deliberately does not occlude the mouse: the drop handler lives on the
/// application root, which would stop being hovered (and therefore stop
/// receiving `on_drop`) if this overlay blocked its hitbox.
pub fn drag_overlay(overlay: DragOverlay) -> impl IntoElement {
    let DragOverlay {
        hover,
        leaving,
        transition,
    } = overlay;
    let (border, background, title, hint, hint_color) = match hover {
        DragHover::Accepted { name: Some(name) } => (
            rgb(ACCENT),
            rgb(PANEL_DRAGGING),
            format!("Drop to import “{name}”"),
            Some("Release the file to start the import.".to_string()),
            MUTED,
        ),
        DragHover::Accepted { name: None } => (
            rgb(ACCENT),
            rgb(PANEL_DRAGGING),
            "Drop the image to import it".to_string(),
            Some("Release the file to start the import.".to_string()),
            MUTED,
        ),
        DragHover::Rejected { name } => (
            rgb(ERROR_BORDER),
            rgb(ERROR_BG),
            format!("“{name}” is not a supported image"),
            Some("Use a PNG, JPEG, WebP, GIF, BMP, TIFF, ICO, PNM, QOI or TGA file.".to_string()),
            ERROR_TEXT,
        ),
        DragHover::Busy => (
            rgb(DISABLED),
            rgb(PANEL),
            "Importing…".to_string(),
            Some("Wait for the current import to finish, then drop the file again.".to_string()),
            MUTED,
        ),
    };
    // The card appears with the neutral colour of the import screen's drop
    // zone and tints towards its state colour. Only the colours animate, so
    // the text is opaque and readable from the first frame.
    let border_from = rgb(BORDER);
    let background_from = rgb(PANEL);

    let card = div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        // Same size and border style as the drop zone of the import screen.
        .w_full()
        .max_w(px(760.))
        .h(px(420.))
        .px_8()
        .rounded_xl()
        .border_1()
        .border_dashed()
        .border_color(border)
        .bg(background)
        .text_center()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_3()
                // On exit the card stays opaque while it fades, so hiding the
                // text up front keeps it from overlapping the text below.
                .opacity(if leaving { 0.0 } else { 1.0 })
                .child(div().text_2xl().text_color(rgb(TEXT)).child(title))
                .when_some(hint, |content, hint| {
                    content.child(div().text_sm().text_color(rgb(hint_color)).child(hint))
                }),
        )
        .with_animation(
            format!("drag-overlay-card-{transition}"),
            Animation::new(DRAG_OVERLAY_FADE).with_easing(ease_out_quint()),
            move |card, progress| {
                if leaving {
                    card.opacity(1.0 - progress)
                } else {
                    card.border_color(Rgba::interpolate(border_from, border, progress))
                        .bg(Rgba::interpolate(background_from, background, progress))
                }
            },
        );

    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        // Only the scrim and the card fade. Animating the text would show it
        // over the text of the screen below.
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .bg(rgba(0x0b1020e6))
                .with_animation(
                    format!("drag-overlay-scrim-{transition}"),
                    Animation::new(DRAG_OVERLAY_FADE).with_easing(ease_out_quint()),
                    move |scrim, progress| {
                        scrim.opacity(if leaving { 1.0 - progress } else { progress })
                    },
                ),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                // Match the import screen padding: the title bar, the screen's
                // own padding and the rest below the card.
                .p_8()
                .pt(title_bar::HEIGHT + px(32.))
                .child(card),
        )
}
