//! Mask over the image preview showing the ellipse the computation draws.
//!
//! The computation inscribes the ellipse in the whole image, so everything
//! outside it is never drawn. This overlay darkens that area while leaving the
//! inside of the ellipse transparent.

use gpui::{
    Bounds, DevicePixels, FillOptions, FillRule, IntoElement, ObjectFit, PathBuilder, PathStyle,
    Pixels, Point, Size, Styled, canvas, point, px, rgba,
};

use super::PREVIEW_MASK;

/// Overlays a dark mask with an elliptical hole matching the image bounds.
///
/// `image_size` must be the size of the image as it is handed to the renderer,
/// so the mask lines up with the pixels actually painted. The element is
/// absolutely positioned and fills its parent.
pub fn ellipse_overlay(image_size: Size<DevicePixels>) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let image = ObjectFit::Contain.get_bounds(bounds, image_size);
            if image.size.width <= px(0.) || image.size.height <= px(0.) {
                return;
            }

            let mut builder = PathBuilder::fill().with_style(PathStyle::Fill(FillOptions {
                fill_rule: FillRule::EvenOdd,
            }));
            // Outer rectangle plus the elliptical hole. With the even-odd rule
            // only the region between both contours is filled.
            add_rect(&mut builder, bounds);
            add_ellipse(&mut builder, image.center(), image.size);

            if let Ok(path) = builder.build() {
                window.paint_path(path, rgba(PREVIEW_MASK));
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
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
    // Distance from an axis endpoint to its control point for a cubic Bézier
    // approximation of a quarter arc.
    const HANDLE: f32 = 0.552_284_8;

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
