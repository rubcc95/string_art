//! Window controls shared by the platforms where the application draws them.
//!
//! Linux and Windows each draw their own buttons, but the glyphs and the hover
//! fade are the same. This module owns that shared drawing; the platform modules
//! only provide the button chrome around it (size, hover fill shape, colours and
//! the platform-specific window-control areas).

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, Div, ElementId, Entity, InteractiveElement as _,
    Interpolate as _, IntoElement, ParentElement as _, PathBuilder, Pixels, Point, Rgba,
    Styled as _, Window, canvas, div, ease_out_quint, point, px, rgb, rgba,
};

use crate::ui::TEXT;

/// Duration of the window controls' hover fade.
pub const HOVER_FADE: Duration = Duration::from_millis(500);

/// Square every control glyph is drawn in.
const GLYPH_SIZE: Pixels = px(16.);
/// Stroke width, shared by every glyph so they keep the same visual weight.
const STROKE: Pixels = px(1.5);
/// Inset of the 10 px shapes inside the glyph square.
const INSET: f32 = 3.;
/// Position of the centred horizontal/vertical middle of the glyph square.
const MIDDLE: f32 = 8.;

/// The window control glyphs, drawn as vector paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    Minimize,
    Maximize,
    Restore,
    Close,
}

/// Draws one control glyph with the shared stroke and geometry.
pub fn glyph(glyph: Glyph, color: Rgba) -> AnyElement {
    stroke_icon(color, move |builder, origin| match glyph {
        Glyph::Minimize => {
            builder.move_to(at(origin, INSET, MIDDLE));
            builder.line_to(at(origin, 13., MIDDLE));
        }
        Glyph::Maximize => rect(builder, origin, INSET, INSET, 13., 13.),
        Glyph::Restore => {
            builder.move_to(at(origin, 6., INSET));
            builder.line_to(at(origin, 13., INSET));
            builder.line_to(at(origin, 13., 10.));
            rect(builder, origin, INSET, 6., 10., 13.);
        }
        Glyph::Close => {
            builder.move_to(at(origin, INSET, INSET));
            builder.line_to(at(origin, 13., 13.));
            builder.move_to(at(origin, 13., INSET));
            builder.line_to(at(origin, INSET, 13.));
        }
    })
}

/// The close glyph, turning white while its button is hovered.
pub fn close_icon() -> AnyElement {
    // Canvas paths cannot be recoloured on hover, so the white shape is a
    // duplicate that fades in over the base one.
    let hover = div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .opacity(0.)
        .group_hover("window-control-close", |style| style.opacity(1.))
        .child(glyph(Glyph::Close, rgb(0xffffff)));

    div()
        .relative()
        .size(GLYPH_SIZE)
        .child(glyph(Glyph::Close, rgb(TEXT)))
        .child(hover)
        .into_any_element()
}

/// Hover state of one window control.
pub struct HoverState {
    state: Entity<Option<bool>>,
}

/// Per-control hover state, kept in window element state.
pub fn use_hover(key: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> HoverState {
    HoverState {
        state: window.use_keyed_state(key, cx, |_, _| None),
    }
}

impl HoverState {
    pub fn get(&self, cx: &App) -> Option<bool> {
        *self.state.read(cx)
    }

    pub fn listener(&self) -> impl Fn(&bool, &mut Window, &mut App) + 'static {
        let state = self.state.clone();
        move |hovered, _, cx| {
            state.update(cx, |slot, cx| {
                *slot = Some(*hovered);
                cx.notify();
            });
        }
    }
}

/// Hover fill that fades in and out over [`HOVER_FADE`].
pub fn hover_fill(
    id: impl Into<ElementId>,
    hovered: Option<bool>,
    color: Rgba,
    element: Div,
) -> AnyElement {
    let clear = rgba(0x00000000);
    let Some(hovered) = hovered else {
        return element.bg(clear).into_any_element();
    };
    let id: ElementId = id.into();
    let (from, to) = if hovered {
        (clear, color)
    } else {
        (color, clear)
    };
    element
        .bg(from)
        .with_animation(
            (id, if hovered { "in" } else { "out" }),
            Animation::new(HOVER_FADE).with_easing(ease_out_quint()),
            move |element, delta| element.bg(Rgba::interpolate(from, to, delta)),
        )
        .into_any_element()
}

fn stroke_icon(
    color: Rgba,
    draw: impl Fn(&mut PathBuilder, Point<Pixels>) + 'static,
) -> AnyElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let mut builder = PathBuilder::stroke(STROKE);
            draw(&mut builder, bounds.origin);
            if let Ok(path) = builder.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(GLYPH_SIZE)
    .into_any_element()
}

fn at(origin: Point<Pixels>, x: f32, y: f32) -> Point<Pixels> {
    origin + point(px(x), px(y))
}

fn rect(
    builder: &mut PathBuilder,
    origin: Point<Pixels>,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
) {
    builder.move_to(at(origin, left, top));
    builder.line_to(at(origin, right, top));
    builder.line_to(at(origin, right, bottom));
    builder.line_to(at(origin, left, bottom));
    builder.line_to(at(origin, left, top));
}
