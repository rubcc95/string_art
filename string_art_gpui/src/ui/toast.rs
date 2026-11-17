//! Floating messages shown above the application content.
//!
//! Toasts are global: any part of the application creates one with [`Toast`]
//! and the view owning the toast layer decides how to show it. The
//! [`ToastStyle`] and [`ToastIcon`] presets pair a colour with an icon for each
//! common kind of message.

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, Bounds, Div, FontWeight, IntoElement, Path,
    PathBuilder, Pixels, Point, Stateful, canvas, div, ease_out_quint, point, prelude::*, px, rgb,
};

use super::{ACCENT, ERROR_BG, ERROR_BORDER, ERROR_TEXT};

/// How long a toast stays on screen before it closes on its own.
pub const TOAST_DURATION: Duration = Duration::from_secs(5);
/// Duration of both the entry and the exit of a toast.
pub const TOAST_FADE: Duration = Duration::from_millis(180);

/// Colours of a toast card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToastStyle {
    pub bg: u32,
    pub border: u32,
    pub text: u32,
    pub icon: u32,
}

impl ToastStyle {
    pub const ERROR: Self = Self {
        bg: ERROR_BG,
        border: ERROR_BORDER,
        text: ERROR_TEXT,
        icon: ERROR_TEXT,
    };
    #[allow(dead_code)] // The presets are ready for future messages.
    pub const WARNING: Self = Self {
        bg: 0x2a2011,
        border: 0x7a5a20,
        text: 0xf0c674,
        icon: 0xf0c674,
    };
    #[allow(dead_code)] // The presets are ready for future messages.
    pub const INFO: Self = Self {
        bg: 0x16233a,
        border: 0x2f5ea8,
        text: 0xa9c7ff,
        icon: ACCENT,
    };
    #[allow(dead_code)] // The presets are ready for future messages.
    pub const SUCCESS: Self = Self {
        bg: 0x14271c,
        border: 0x2f7a4a,
        text: 0x9be0b4,
        icon: 0x9be0b4,
    };
}

/// The drawing shown in the toast, paired with a [`ToastStyle`] by the presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Only variants used by callers are constructed.
pub enum ToastIcon {
    Error,
    Warning,
    Info,
    Success,
}

/// A message shown to the user.
#[derive(Clone, Debug)]
pub struct Toast {
    pub color: ToastStyle,
    pub title: String,
    pub description: Option<String>,
    pub icon: ToastIcon,
}

impl Toast {
    /// Creates a toast with an explicit colour and icon.
    pub fn new(
        color: ToastStyle,
        title: impl Into<String>,
        description: Option<String>,
        icon: ToastIcon,
    ) -> Self {
        Self {
            color,
            title: title.into(),
            description,
            icon,
        }
    }

    pub fn error(title: impl Into<String>, description: Option<String>) -> Self {
        Self::new(ToastStyle::ERROR, title, description, ToastIcon::Error)
    }

    #[allow(dead_code)] // The presets are ready for future messages.
    pub fn warning(title: impl Into<String>, description: Option<String>) -> Self {
        Self::new(ToastStyle::WARNING, title, description, ToastIcon::Warning)
    }

    #[allow(dead_code)] // The presets are ready for future messages.
    pub fn info(title: impl Into<String>, description: Option<String>) -> Self {
        Self::new(ToastStyle::INFO, title, description, ToastIcon::Info)
    }

    #[allow(dead_code)] // The presets are ready for future messages.
    pub fn success(title: impl Into<String>, description: Option<String>) -> Self {
        Self::new(ToastStyle::SUCCESS, title, description, ToastIcon::Success)
    }
}

/// A toast owned by the application, with the state needed to animate it out.
#[derive(Clone, Debug)]
pub struct ToastEntry {
    pub id: u64,
    pub toast: Toast,
    /// Whether the toast is fading out and will be removed afterwards.
    pub leaving: bool,
}

/// Stack of toasts in the top-right corner, below the title bar.
pub fn toast_layer(children: impl IntoIterator<Item = AnyElement>) -> impl IntoElement {
    div()
        .absolute()
        .top(super::title_bar::HEIGHT + px(12.))
        .right(px(12.))
        .flex()
        .flex_col()
        .items_end()
        .gap_2()
        .children(children)
}

/// One toast card. The caller attaches the click handler that dismisses it.
pub fn toast_card(entry: &ToastEntry) -> Stateful<Div> {
    let id = entry.id;
    let leaving = entry.leaving;
    let Toast {
        color,
        title,
        description,
        icon,
    } = entry.toast.clone();
    let animation = format!("toast-{id}-{}", if leaving { "out" } else { "in" });

    div().id(("toast", id)).cursor_pointer().child(
        div()
            .w(px(360.))
            .flex()
            .items_start()
            .gap_3()
            .rounded_lg()
            .border_1()
            .border_color(rgb(color.border))
            .bg(rgb(color.bg))
            .px_4()
            .py_3()
            .child(toast_icon(icon, color.icon, color.bg))
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(color.text))
                            .child(title),
                    )
                    .when_some(description, |column, description| {
                        column.child(
                            div()
                                .text_xs()
                                .text_color(rgb(color.text))
                                .opacity(0.8)
                                .child(description),
                        )
                    }),
            )
            .with_animation(
                animation,
                Animation::new(TOAST_FADE).with_easing(ease_out_quint()),
                move |card, progress| card.opacity(if leaving { 1.0 - progress } else { progress }),
            ),
    )
}

/// Draws the icon with vector paths so it does not depend on any font or asset.
fn toast_icon(icon: ToastIcon, foreground: u32, background: u32) -> impl IntoElement {
    div().flex_none().size(px(20.)).child(
        canvas(
            move |bounds, _, _| icon_paths(bounds, icon, foreground, background),
            move |_, paths, window, _| {
                for (path, color) in paths {
                    window.paint_path(path, rgb(color));
                }
            },
        )
        .size_full(),
    )
}

/// Builds the filled paths of an icon, in paint order.
fn icon_paths(
    bounds: Bounds<Pixels>,
    icon: ToastIcon,
    foreground: u32,
    background: u32,
) -> Vec<(Path, u32)> {
    let size = bounds.size.width.min(bounds.size.height);
    let origin = point(
        bounds.origin.x + (bounds.size.width - size) / 2.,
        bounds.origin.y + (bounds.size.height - size) / 2.,
    );
    let at = |x: f32, y: f32| point(origin.x + size * (x / 20.), origin.y + size * (y / 20.));
    let stroke = size * (2.0 / 20.0);
    let mut paths = Vec::new();

    match icon {
        ToastIcon::Error => {
            let mut circle = PathBuilder::fill();
            add_circle(&mut circle, at(10., 10.), size * (9. / 20.));
            push_path(&mut paths, circle, foreground);

            let mut cross = PathBuilder::stroke(stroke);
            cross.move_to(at(6.5, 6.5));
            cross.line_to(at(13.5, 13.5));
            cross.move_to(at(13.5, 6.5));
            cross.line_to(at(6.5, 13.5));
            push_path(&mut paths, cross, background);
        }
        ToastIcon::Warning => {
            let mut triangle = PathBuilder::fill();
            triangle.add_polygon(&[at(10., 2.2), at(18.2, 17.2), at(1.8, 17.2)], true);
            push_path(&mut paths, triangle, foreground);

            let mut bar = PathBuilder::stroke(stroke);
            bar.move_to(at(10., 7.2));
            bar.line_to(at(10., 12.2));
            push_path(&mut paths, bar, background);

            let mut dot = PathBuilder::fill();
            add_circle(&mut dot, at(10., 14.8), size * (1.1 / 20.));
            push_path(&mut paths, dot, background);
        }
        ToastIcon::Info => {
            let mut circle = PathBuilder::fill();
            add_circle(&mut circle, at(10., 10.), size * (9. / 20.));
            push_path(&mut paths, circle, foreground);

            let mut dot = PathBuilder::fill();
            add_circle(&mut dot, at(10., 5.7), size * (1.15 / 20.));
            push_path(&mut paths, dot, background);

            let mut bar = PathBuilder::stroke(stroke);
            bar.move_to(at(10., 8.6));
            bar.line_to(at(10., 14.4));
            push_path(&mut paths, bar, background);
        }
        ToastIcon::Success => {
            let mut circle = PathBuilder::fill();
            add_circle(&mut circle, at(10., 10.), size * (9. / 20.));
            push_path(&mut paths, circle, foreground);

            let mut check = PathBuilder::stroke(stroke);
            check.move_to(at(5.7, 10.3));
            check.line_to(at(8.8, 13.3));
            check.line_to(at(14.4, 7.2));
            push_path(&mut paths, check, background);
        }
    }

    paths
}

fn push_path(paths: &mut Vec<(Path, u32)>, builder: PathBuilder, color: u32) {
    if let Ok(path) = builder.build() {
        paths.push((path, color));
    }
}

/// Adds a circle to a path using four cubic Bézier segments.
fn add_circle(builder: &mut PathBuilder, center: Point<Pixels>, radius: Pixels) {
    let handle = radius * 0.552_284_8;
    builder.move_to(point(center.x + radius, center.y));
    builder.cubic_bezier_to(
        point(center.x, center.y + radius),
        point(center.x + radius, center.y + handle),
        point(center.x + handle, center.y + radius),
    );
    builder.cubic_bezier_to(
        point(center.x - radius, center.y),
        point(center.x - handle, center.y + radius),
        point(center.x - radius, center.y + handle),
    );
    builder.cubic_bezier_to(
        point(center.x, center.y - radius),
        point(center.x - radius, center.y - handle),
        point(center.x - handle, center.y - radius),
    );
    builder.cubic_bezier_to(
        point(center.x + radius, center.y),
        point(center.x + handle, center.y - radius),
        point(center.x + radius, center.y - handle),
    );
    builder.close();
}
