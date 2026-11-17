//! Full-window hint shown while a file is being dragged over the application.

use gpui::{
    Animation, AnimationExt as _, Interpolate as _, IntoElement, Rgba, div, ease_out_quint,
    prelude::*, px, rgb, rgba,
};

use super::{
    ACCENT, BORDER, DRAG_OVERLAY_FADE, ERROR_BG, ERROR_BORDER, ERROR_TEXT, MUTED, PANEL,
    PANEL_DRAGGING, TEXT, title_bar,
};

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
            rgb(super::DISABLED),
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
