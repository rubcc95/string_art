//! Ellipse state for one workspace: frame, hover, arrows and drag.

use gpui::{Bounds, Pixels, Point};

use crate::ui::{EllipseDragKind, EllipseFrame, EllipsePointerEvent, ResizeHandle};

/// Smallest ellipse size, as a fraction of the image.
const MIN_SIZE: f32 = 0.03;

/// Everything the overlay needs to know about this workspace's ellipse.
pub(crate) struct EllipseState {
    pub frame: EllipseFrame,
    pub hover: Option<ResizeHandle>,
    pub arrows_seen: [bool; 8],
    drag: Option<EllipseDrag>,
}

impl Default for EllipseState {
    fn default() -> Self {
        Self {
            frame: EllipseFrame::FULL,
            hover: None,
            arrows_seen: [false; 8],
            drag: None,
        }
    }
}

impl EllipseState {
    /// Applies a pointer event. Returns `true` when the view should repaint.
    pub fn pointer(&mut self, event: &EllipsePointerEvent) -> bool {
        match *event {
            EllipsePointerEvent::Down { kind, position } => {
                self.drag = Some(EllipseDrag::new(kind, position, self.frame));
                let hover = match kind {
                    EllipseDragKind::Resize(handle) => {
                        self.arrows_seen[handle.index()] = true;
                        Some(handle)
                    }
                    EllipseDragKind::Move => None,
                };
                let changed = self.hover != hover;
                self.hover = hover;
                changed
            }
            EllipsePointerEvent::Move {
                position,
                image_bounds,
            } => {
                let Some(drag) = self.drag else {
                    return false;
                };
                let frame = drag.update(position, image_bounds);
                let changed = frame != self.frame;
                self.frame = frame;
                changed
            }
            EllipsePointerEvent::Hover(hover) => {
                self.drag = None;
                if let Some(handle) = hover {
                    self.arrows_seen[handle.index()] = true;
                }
                let changed = self.hover != hover;
                self.hover = hover;
                changed
            }
            EllipsePointerEvent::Up => {
                self.drag = None;
                false
            }
        }
    }
}

/// One drag in progress. The frame at the start of the drag is kept so every
/// update is computed from the original shape instead of accumulating error.
#[derive(Clone, Copy, Debug)]
struct EllipseDrag {
    kind: EllipseDragKind,
    start_pointer: Point<Pixels>,
    start_frame: EllipseFrame,
}

impl EllipseDrag {
    fn new(kind: EllipseDragKind, position: Point<Pixels>, frame: EllipseFrame) -> Self {
        Self {
            kind,
            start_pointer: position,
            start_frame: frame,
        }
    }

    fn update(self, position: Point<Pixels>, image_bounds: Bounds<Pixels>) -> EllipseFrame {
        let width = image_bounds.size.width.as_f32();
        let height = image_bounds.size.height.as_f32();
        if width <= 0. || height <= 0. {
            return self.start_frame;
        }

        let dx = (position.x.as_f32() - self.start_pointer.x.as_f32()) / width;
        let dy = (position.y.as_f32() - self.start_pointer.y.as_f32()) / height;

        match self.kind {
            EllipseDragKind::Move => move_frame(self.start_frame, dx, dy),
            EllipseDragKind::Resize(handle) => resize_frame(self.start_frame, handle, dx, dy),
        }
    }
}

fn move_frame(frame: EllipseFrame, dx: f32, dy: f32) -> EllipseFrame {
    clamp(EllipseFrame {
        x: frame.x + dx,
        y: frame.y + dy,
        ..frame
    })
}

fn resize_frame(frame: EllipseFrame, handle: ResizeHandle, dx: f32, dy: f32) -> EllipseFrame {
    let mut left = frame.x;
    let mut top = frame.y;
    let mut right = frame.x + frame.width;
    let mut bottom = frame.y + frame.height;

    if matches!(
        handle,
        ResizeHandle::TopLeft | ResizeHandle::Left | ResizeHandle::BottomLeft
    ) {
        left = (left + dx).clamp(0., right - MIN_SIZE);
    }
    if matches!(
        handle,
        ResizeHandle::TopRight | ResizeHandle::Right | ResizeHandle::BottomRight
    ) {
        right = (right + dx).clamp(left + MIN_SIZE, 1.);
    }
    if matches!(
        handle,
        ResizeHandle::TopLeft | ResizeHandle::Top | ResizeHandle::TopRight
    ) {
        top = (top + dy).clamp(0., bottom - MIN_SIZE);
    }
    if matches!(
        handle,
        ResizeHandle::BottomLeft | ResizeHandle::Bottom | ResizeHandle::BottomRight
    ) {
        bottom = (bottom + dy).clamp(top + MIN_SIZE, 1.);
    }

    EllipseFrame {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

fn clamp(frame: EllipseFrame) -> EllipseFrame {
    let width = frame.width.clamp(MIN_SIZE, 1.);
    let height = frame.height.clamp(MIN_SIZE, 1.);
    EllipseFrame {
        x: frame.x.clamp(0., 1. - width),
        y: frame.y.clamp(0., 1. - height),
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(x: f32, y: f32, width: f32, height: f32) -> EllipseFrame {
        EllipseFrame {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn moving_clamps_the_ellipse_to_the_image() {
        assert_eq!(
            move_frame(frame(0.2, 0.2, 0.4, 0.4), 0.8, 0.8),
            frame(0.6, 0.6, 0.4, 0.4)
        );
        assert_eq!(
            move_frame(frame(0.2, 0.2, 0.4, 0.4), -0.8, -0.8),
            frame(0., 0., 0.4, 0.4)
        );
    }

    #[test]
    fn resizing_keeps_the_opposite_edge_fixed_and_stays_inside() {
        let start = frame(0.2, 0.2, 0.4, 0.4);
        assert_eq!(
            resize_frame(start, ResizeHandle::TopLeft, -1., -1.),
            frame(0., 0., 0.6, 0.6)
        );
        assert_eq!(
            resize_frame(start, ResizeHandle::BottomRight, 1., 1.),
            frame(0.2, 0.2, 0.8, 0.8)
        );
    }

    #[test]
    fn resizing_never_collapses_the_ellipse() {
        let start = frame(0.2, 0.2, 0.4, 0.4);
        let resized = resize_frame(start, ResizeHandle::TopLeft, 10., 10.);
        assert!((resized.width - MIN_SIZE).abs() < 1e-5);
        assert!((resized.height - MIN_SIZE).abs() < 1e-5);
        assert!((resized.x + resized.width - 0.6).abs() < 1e-5);
        assert!((resized.y + resized.height - 0.6).abs() < 1e-5);
    }
}
