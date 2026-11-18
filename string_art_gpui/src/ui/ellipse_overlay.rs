//! Mask over the image preview showing the ellipse the computation draws.
//!
//! The computation inscribes the ellipse in the image, so everything outside it
//! is never drawn. This overlay darkens that area while leaving the inside of
//! the ellipse transparent, marks where the nails will be placed and provides
//! the resize arrows used to move and resize the ellipse.

use std::f32::consts::TAU;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, Bounds, DevicePixels, FillOptions, FillRule,
    IntoElement, LineCap, LineJoin, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ObjectFit, Path, PathBuilder, PathStyle, Pixels, Point, Size, StrokeOptions, Styled, Window,
    canvas, div, ease_out_quint, point, prelude::*, px, rgba,
};

use super::PREVIEW_MASK;

/// Smallest radius, in screen pixels, at which a nail is still visible when the
/// image is much larger than its preview.
const MIN_NAIL_RADIUS: f32 = 1.5;
/// Colour of the nail markers.
const NAIL: u32 = 0xe7e7eacc;
/// Half-width of the invisible resize ring around the ellipse outline.
const EDGE_HIT_SIZE: f32 = 20.;
/// Dark halo drawn under the resize arrow.
const ARROW_HALO: u32 = 0x0f0f12e6;
/// Colour of the resize arrow.
const ARROW: u32 = 0xffffffff;
/// Length of the double-headed resize arrow.
const ARROW_LENGTH: f32 = 28.;
/// Length of each arrowhead.
const ARROW_HEAD: f32 = 8.;
/// Half the width of each arrowhead.
const ARROW_SPREAD: f32 = 5.;
/// Duration of the arrow fade-in and fade-out.
const ARROW_FADE: Duration = Duration::from_millis(500);

/// Position and size of the ellipse, as fractions of the displayed image.
///
/// [`EllipseFrame::FULL`] makes the ellipse fill the whole image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EllipseFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl EllipseFrame {
    /// The ellipse inscribed in the whole image.
    pub const FULL: Self = Self {
        x: 0.,
        y: 0.,
        width: 1.,
        height: 1.,
    };

    fn bounds(self, image: Bounds<Pixels>) -> Bounds<Pixels> {
        Bounds {
            origin: point(
                image.origin.x + image.size.width * self.x,
                image.origin.y + image.size.height * self.y,
            ),
            size: Size {
                width: image.size.width * self.width,
                height: image.size.height * self.height,
            },
        }
    }
}

/// One of the eight resize arrows around the ellipse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

/// Order matches the eight anchors at 45° intervals, starting on the right.
const ANCHOR_HANDLES: [ResizeHandle; 8] = [
    ResizeHandle::Right,
    ResizeHandle::BottomRight,
    ResizeHandle::Bottom,
    ResizeHandle::BottomLeft,
    ResizeHandle::Left,
    ResizeHandle::TopLeft,
    ResizeHandle::Top,
    ResizeHandle::TopRight,
];

impl ResizeHandle {
    /// Index of this arrow in [`ANCHOR_HANDLES`].
    pub fn index(self) -> usize {
        ANCHOR_HANDLES
            .iter()
            .position(|candidate| *candidate == self)
            .expect("all resize handles are in ANCHOR_HANDLES")
    }
}

/// What a drag on the ellipse is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EllipseDragKind {
    Move,
    Resize(ResizeHandle),
}

/// Pointer event emitted by the overlay. Coordinates are in window pixels.
#[derive(Clone, Copy, Debug)]
pub enum EllipsePointerEvent {
    Down {
        kind: EllipseDragKind,
        position: Point<Pixels>,
    },
    Move {
        position: Point<Pixels>,
        image_bounds: Bounds<Pixels>,
    },
    Hover(Option<ResizeHandle>),
    Up,
}

/// Everything needed to paint the base overlay, built once per frame.
struct OverlayPaths {
    mask: Path,
    nails: Option<Path>,
    image: Bounds<Pixels>,
    ellipse: Bounds<Pixels>,
}

/// The halo and white stroke of one resize arrow.
struct ArrowPaths {
    halo: Path,
    arrow: Path,
}

/// Overlays a dark mask with an elliptical hole, marks the position of every
/// nail and keeps the eight resize arrows mounted with independent fades.
///
/// `image_size` must be the size of the image as it is handed to the renderer,
/// so the mask lines up with the pixels actually painted. `source_size` is the
/// size of the image the computation works on, used to scale the nail radius.
/// The element fills its parent, which positions it over the image.
pub fn ellipse_overlay(
    image_size: Size<DevicePixels>,
    source_size: Size<Pixels>,
    frame: EllipseFrame,
    hover: Option<ResizeHandle>,
    arrows_seen: [bool; 8],
    nail_count: usize,
    nail_radius_px: f32,
    on_pointer: impl Fn(&EllipsePointerEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let mut root = div()
        .relative()
        .size_full()
        .child(base_overlay(
            image_size,
            source_size,
            frame,
            nail_count,
            nail_radius_px,
            on_pointer,
        ));

    for (index, handle) in ANCHOR_HANDLES.into_iter().enumerate() {
        root = root.child(arrow_layer(
            image_size,
            frame,
            handle,
            index,
            hover == Some(handle),
            arrows_seen[index],
        ));
    }

    root
}

fn base_overlay(
    image_size: Size<DevicePixels>,
    source_size: Size<Pixels>,
    frame: EllipseFrame,
    nail_count: usize,
    nail_radius_px: f32,
    on_pointer: impl Fn(&EllipsePointerEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let paint_pointer = Rc::new(on_pointer);

    canvas(
        move |bounds, _, _| {
            let image = ObjectFit::Contain.get_bounds(bounds, image_size);
            if image.size.width <= px(0.) || image.size.height <= px(0.) {
                return None;
            }
            let ellipse = frame.bounds(image);
            if ellipse.size.width <= px(0.) || ellipse.size.height <= px(0.) {
                return None;
            }

            let mut mask = PathBuilder::fill().with_style(PathStyle::Fill(FillOptions {
                fill_rule: FillRule::EvenOdd,
            }));
            // Outer rectangle plus the elliptical hole. With the even-odd rule
            // only the region between both contours is filled.
            add_rect(&mut mask, bounds);
            add_ellipse(&mut mask, ellipse.center(), ellipse.size);

            Some(OverlayPaths {
                mask: mask.build().ok()?,
                nails: nail_path(ellipse, source_size, nail_count, nail_radius_px),
                image,
                ellipse,
            })
        },
        move |_, paths, window, _| {
            let Some(OverlayPaths {
                mask,
                nails,
                image,
                ellipse,
            }) = paths
            else {
                return;
            };

            let down_handler = paint_pointer.clone();
            window.on_mouse_event(move |event: &MouseDownEvent, _, window, cx| {
                if event.button != MouseButton::Left {
                    return;
                }
                let Some(kind) = hit_test(event.position, ellipse) else {
                    return;
                };
                down_handler(
                    &EllipsePointerEvent::Down {
                        kind,
                        position: event.position,
                    },
                    window,
                    cx,
                );
            });

            let move_handler = paint_pointer.clone();
            window.on_mouse_event(move |event: &MouseMoveEvent, _, window, cx| {
                let pointer_event = if event.dragging() {
                    EllipsePointerEvent::Move {
                        position: event.position,
                        image_bounds: image,
                    }
                } else {
                    EllipsePointerEvent::Hover(resize_target(event.position, ellipse))
                };
                move_handler(&pointer_event, window, cx);
            });

            let up_handler = paint_pointer.clone();
            window.on_mouse_event(move |_: &MouseUpEvent, _, window, cx| {
                up_handler(&EllipsePointerEvent::Up, window, cx);
            });

            window.paint_path(mask, rgba(PREVIEW_MASK));
            if let Some(nails) = nails {
                window.paint_path(nails, rgba(NAIL));
            }
        },
    )
    .size_full()
}

fn arrow_layer(
    image_size: Size<DevicePixels>,
    frame: EllipseFrame,
    handle: ResizeHandle,
    index: usize,
    active: bool,
    seen: bool,
) -> AnyElement {
    let layer = div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .child(
            canvas(
                move |bounds, _, _| {
                    let image = ObjectFit::Contain.get_bounds(bounds, image_size);
                    if image.size.width <= px(0.) || image.size.height <= px(0.) {
                        return None;
                    }
                    let ellipse = frame.bounds(image);
                    if ellipse.size.width <= px(0.) || ellipse.size.height <= px(0.) {
                        return None;
                    }
                    arrow_paths(ellipse, handle)
                },
                |_, paths, window, _| {
                    let Some(paths) = paths else {
                        return;
                    };
                    window.paint_path(paths.halo, rgba(ARROW_HALO));
                    window.paint_path(paths.arrow, rgba(ARROW));
                },
            )
            .size_full(),
        );

    if !seen {
        return layer.opacity(0.).into_any_element();
    }

    let (from, to) = if active { (0., 1.) } else { (1., 0.) };
    let id = if active {
        ("ellipse-resize-arrow-in", index)
    } else {
        ("ellipse-resize-arrow-out", index)
    };

    layer
        .opacity(from)
        .with_animation(
            id,
            Animation::new(ARROW_FADE).with_easing(ease_out_quint()),
            move |element, delta| element.opacity(from + (to - from) * delta),
        )
        .into_any_element()
}

fn hit_test(position: Point<Pixels>, ellipse: Bounds<Pixels>) -> Option<EllipseDragKind> {
    if let Some(handle) = resize_target(position, ellipse) {
        return Some(EllipseDragKind::Resize(handle));
    }

    let center = ellipse.center();
    let rx = ellipse.size.width.as_f32() / 2.;
    let ry = ellipse.size.height.as_f32() / 2.;
    if rx <= 0. || ry <= 0. {
        return None;
    }
    let u = (position.x.as_f32() - center.x.as_f32()) / rx;
    let v = (position.y.as_f32() - center.y.as_f32()) / ry;
    (u * u + v * v <= 1.).then_some(EllipseDragKind::Move)
}

/// Returns the nearest resize anchor when the pointer is within 20 px of the
/// ellipse on either side of its outline.
fn resize_target(position: Point<Pixels>, ellipse: Bounds<Pixels>) -> Option<ResizeHandle> {
    let center = ellipse.center();
    let rx = ellipse.size.width.as_f32() / 2.;
    let ry = ellipse.size.height.as_f32() / 2.;
    if rx <= 0. || ry <= 0. {
        return None;
    }

    let x = position.x.as_f32() - center.x.as_f32();
    let y = position.y.as_f32() - center.y.as_f32();
    let u = x / rx;
    let v = y / ry;

    // First-order approximation of the distance to the ellipse using the
    // gradient of its implicit equation. It is accurate close to the outline.
    let f = u * u + v * v - 1.;
    let gx = 2. * u / rx;
    let gy = 2. * v / ry;
    let gradient = (gx * gx + gy * gy).sqrt();
    if gradient <= f32::EPSILON {
        return None;
    }
    let distance = f.abs() / gradient;
    let band = EDGE_HIT_SIZE.min(rx.min(ry) * 0.5);
    if distance > band {
        return None;
    }

    let mut nearest = ANCHOR_HANDLES[0];
    let mut nearest_distance = f32::INFINITY;
    for (index, handle) in ANCHOR_HANDLES.into_iter().enumerate() {
        let angle = TAU * index as f32 / ANCHOR_HANDLES.len() as f32;
        let ax = rx * angle.cos();
        let ay = ry * angle.sin();
        let dx = x - ax;
        let dy = y - ay;
        let distance = dx * dx + dy * dy;
        if distance < nearest_distance {
            nearest_distance = distance;
            nearest = handle;
        }
    }
    Some(nearest)
}

fn arrow_paths(ellipse: Bounds<Pixels>, handle: ResizeHandle) -> Option<ArrowPaths> {
    let width = ellipse.size.width.as_f32();
    let height = ellipse.size.height.as_f32();
    if width <= 0. || height <= 0. {
        return None;
    }
    let rx = width / 2.;
    let ry = height / 2.;
    let center = ellipse.center();
    let center_x = center.x.as_f32();
    let center_y = center.y.as_f32();

    let index = handle.index();
    let angle = TAU * index as f32 / ANCHOR_HANDLES.len() as f32;
    let (sin, cos) = angle.sin_cos();

    // Anchor on the ellipse itself, oriented along its normal.
    let anchor_x = center_x + rx * cos;
    let anchor_y = center_y + ry * sin;
    let mut dx = cos / rx;
    let mut dy = sin / ry;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return None;
    }
    dx /= -length;
    dy /= -length;

    // The arrow is centred on the ellipse outline: half of it stays inside
    // the ellipse and half stays outside.
    let x = anchor_x;
    let y = anchor_y;

    let mut halo = arrow_stroke(px(5.));
    add_arrow(&mut halo, x, y, dx, dy);
    let mut arrow = arrow_stroke(px(2.25));
    add_arrow(&mut arrow, x, y, dx, dy);

    Some(ArrowPaths {
        halo: halo.build().ok()?,
        arrow: arrow.build().ok()?,
    })
}

fn arrow_stroke(width: Pixels) -> PathBuilder {
    PathBuilder::stroke(width).with_style(PathStyle::Stroke(StrokeOptions {
        line_width: width.as_f32(),
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        miter_limit: 4.,
    }))
}

fn add_arrow(builder: &mut PathBuilder, cx: f32, cy: f32, dx: f32, dy: f32) {
    let half = ARROW_LENGTH / 2.;
    let start = (cx - dx * half, cy - dy * half);
    let end = (cx + dx * half, cy + dy * half);
    let perp = (-dy, dx);

    builder.move_to(point(px(start.0), px(start.1)));
    builder.line_to(point(px(end.0), px(end.1)));

    add_arrow_head(builder, start, (dx, dy), perp);
    add_arrow_head(builder, end, (-dx, -dy), perp);
}

fn add_arrow_head(
    builder: &mut PathBuilder,
    tip: (f32, f32),
    backward: (f32, f32),
    perp: (f32, f32),
) {
    let base = (
        tip.0 + backward.0 * ARROW_HEAD,
        tip.1 + backward.1 * ARROW_HEAD,
    );
    let left = (
        base.0 + perp.0 * ARROW_SPREAD,
        base.1 + perp.1 * ARROW_SPREAD,
    );
    let right = (
        base.0 - perp.0 * ARROW_SPREAD,
        base.1 - perp.1 * ARROW_SPREAD,
    );

    // One connected polyline so the joint at the tip gets a round join instead
    // of two butt-capped segments meeting in a notch.
    builder.move_to(point(px(left.0), px(left.1)));
    builder.line_to(point(px(tip.0), px(tip.1)));
    builder.line_to(point(px(right.0), px(right.1)));
}

fn add_rect(builder: &mut PathBuilder, bounds: Bounds<Pixels>) {
    builder.move_to(bounds.origin);
    builder.line_to(point(bounds.origin.x + bounds.size.width, bounds.origin.y));
    builder.line_to(point(
        bounds.origin.x + bounds.size.width,
        bounds.origin.y + bounds.size.height,
    ));
    builder.line_to(point(bounds.origin.x, bounds.origin.y + bounds.size.height));
    builder.close();
}

/// Adds an ellipse using four cubic Bézier segments.
fn add_ellipse(builder: &mut PathBuilder, center: Point<Pixels>, size: Size<Pixels>) {
    let cx = center.x.as_f32();
    let cy = center.y.as_f32();
    let rx = size.width.as_f32() / 2.;
    let ry = size.height.as_f32() / 2.;
    let hx = rx * HANDLE;
    let hy = ry * HANDLE;
    let at = |x: f32, y: f32| point(px(x), px(y));

    builder.move_to(at(cx + rx, cy));
    builder.cubic_bezier_to(at(cx, cy + ry), at(cx + rx, cy + hy), at(cx + hx, cy + ry));
    builder.cubic_bezier_to(at(cx - rx, cy), at(cx - hx, cy + ry), at(cx - rx, cy + hy));
    builder.cubic_bezier_to(at(cx, cy - ry), at(cx - rx, cy - hy), at(cx - hx, cy - ry));
    builder.cubic_bezier_to(at(cx + rx, cy), at(cx + hx, cy - ry), at(cx + rx, cy - hy));
    builder.close();
}

/// Builds one circle per nail at the same positions `string_art::Ellipse` uses.
fn nail_path(
    ellipse: Bounds<Pixels>,
    source_size: Size<Pixels>,
    nail_count: usize,
    nail_radius_px: f32,
) -> Option<Path> {
    if nail_count == 0 || source_size.width <= px(0.) || source_size.height <= px(0.) {
        return None;
    }

    let source_width = source_size.width.as_f32();
    let source_height = source_size.height.as_f32();
    let ellipse_width = ellipse.size.width.as_f32();
    let ellipse_height = ellipse.size.height.as_f32();
    // The preview may have rounded differently on each axis, so average both
    // scales to keep the nails round.
    let scale = (ellipse_width / source_width + ellipse_height / source_height) / 2.;
    let radius = (nail_radius_px * scale).max(MIN_NAIL_RADIUS);

    let cx = ellipse.origin.x.as_f32() + ellipse_width / 2.;
    let cy = ellipse.origin.y.as_f32() + ellipse_height / 2.;
    let rx = ellipse_width / 2.;
    let ry = ellipse_height / 2.;

    // Circles overlap when the nails are dense, so they must be filled with the
    // non-zero rule to avoid the overlaps becoming holes.
    let mut builder = PathBuilder::fill().with_style(PathStyle::Fill(FillOptions {
        fill_rule: FillRule::NonZero,
    }));
    for i in 0..nail_count {
        let theta = TAU * i as f32 / nail_count as f32;
        let center = point(px(cx + rx * theta.cos()), px(cy + ry * theta.sin()));
        add_circle(&mut builder, center, radius);
    }
    builder.build().ok()
}

/// Distance from an axis endpoint to its control point for a cubic Bézier
/// approximation of a quarter arc.
const HANDLE: f32 = 0.552_284_8;

/// Adds a circle using four cubic Bézier segments.
fn add_circle(builder: &mut PathBuilder, center: Point<Pixels>, radius: f32) {
    let cx = center.x.as_f32();
    let cy = center.y.as_f32();
    let handle = radius * HANDLE;
    let at = |x: f32, y: f32| point(px(x), px(y));

    builder.move_to(at(cx + radius, cy));
    builder.cubic_bezier_to(
        at(cx, cy + radius),
        at(cx + radius, cy + handle),
        at(cx + handle, cy + radius),
    );
    builder.cubic_bezier_to(
        at(cx - radius, cy),
        at(cx - handle, cy + radius),
        at(cx - radius, cy + handle),
    );
    builder.cubic_bezier_to(
        at(cx, cy - radius),
        at(cx - radius, cy - handle),
        at(cx - handle, cy - radius),
    );
    builder.cubic_bezier_to(
        at(cx + radius, cy),
        at(cx + handle, cy - radius),
        at(cx + radius, cy - handle),
    );
    builder.close();
}
