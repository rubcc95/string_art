//! Mask over the image preview showing the ellipse the computation draws.
//!
//! The computation inscribes the ellipse in the whole image, so everything
//! outside it is never drawn. This overlay darkens that area while leaving the
//! inside of the ellipse transparent, and marks where the nails will be placed.

use std::f32::consts::TAU;

use gpui::{
    Bounds, DevicePixels, FillOptions, FillRule, IntoElement, ObjectFit, Path, PathBuilder,
    PathStyle, Pixels, Point, Size, Styled, canvas, point, px, rgba,
};

use super::PREVIEW_MASK;

/// Smallest radius, in screen pixels, at which a nail is still visible when the
/// image is much larger than its preview.
const MIN_NAIL_RADIUS: f32 = 1.5;
/// Colour of the nail markers.
const NAIL: u32 = 0xe7e7eacc;

/// Position and size of the ellipse, as fractions of the displayed image.
///
/// [`EllipseFrame::FULL`] makes the ellipse fill the whole image. Keeping this
/// separate from the image bounds lets the ellipse be moved and resized later.
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

/// Overlays a dark mask with an elliptical hole and marks the position of every
/// nail around the ellipse.
///
/// `image_size` must be the size of the image as it is handed to the renderer,
/// so the mask lines up with the pixels actually painted. `source_size` is the
/// size of the image the computation works on, used to scale the nail radius.
/// The element fills its parent, which positions it over the image.
pub fn ellipse_overlay(
    image_size: Size<DevicePixels>,
    source_size: Size<Pixels>,
    frame: EllipseFrame,
    nail_count: usize,
    nail_radius_px: f32,
) -> impl IntoElement {
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

            let nails = nail_path(ellipse, source_size, nail_count, nail_radius_px);
            Some((mask.build().ok()?, nails))
        },
        |_, paths, window, _| {
            let Some((mask, nails)) = paths else {
                return;
            };
            window.paint_path(mask, rgba(PREVIEW_MASK));
            if let Some(nails) = nails {
                window.paint_path(nails, rgba(NAIL));
            }
        },
    )
    .size_full()
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
