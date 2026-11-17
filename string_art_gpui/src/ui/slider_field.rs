//! Labeled slider with a numeric field for exact values.

use gpui::{
    Context, Entity, EventEmitter, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels,
    SharedString, Subscription, Window, canvas, div, prelude::*, px, relative, rgb,
};

use super::number_input::{NumberInput, NumberInputEvent, NumberInputOptions};
use super::{ACCENT, BG, BORDER, FAINT, TEXT};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderFieldEvent {
    Changed(f64),
}

pub struct SliderField {
    label: SharedString,
    hint: SharedString,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    dragging: bool,
    input: Entity<NumberInput>,
    _input_subscription: Subscription,
}

impl EventEmitter<SliderFieldEvent> for SliderField {}

impl SliderField {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        label: impl Into<SharedString>,
        hint: impl Into<SharedString>,
        suffix: impl Into<SharedString>,
        value: f64,
        min: f64,
        max: f64,
        step: f64,
        decimals: u8,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            NumberInput::new(
                value,
                NumberInputOptions {
                    min,
                    max,
                    decimals,
                    suffix: suffix.into(),
                },
                cx,
            )
        });
        let input_subscription = cx.subscribe(&input, |this, _, event, cx| {
            let NumberInputEvent::Changed(value) = *event;
            this.on_input_changed(value, cx);
        });

        Self {
            label: label.into(),
            hint: hint.into(),
            value: normalize(value, min, max, step),
            min,
            max,
            step,
            dragging: false,
            input,
            _input_subscription: input_subscription,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn set_value(&mut self, value: f64, cx: &mut Context<Self>) {
        self.value = normalize(value, self.min, self.max, self.step);
        let value = self.value;
        self.input
            .update(cx, |input, cx| input.set_value(value, cx));
        cx.notify();
    }

    pub fn set_range(&mut self, min: f64, max: f64, cx: &mut Context<Self>) {
        self.min = min;
        self.max = max;
        let value = normalize(self.value, min, max, self.step);
        self.value = value;
        self.input.update(cx, |input, cx| {
            input.set_range(min, max, cx);
            input.set_value(value, cx);
        });
        cx.notify();
    }

    pub fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| input.commit(window, cx));
        let value = normalize(self.input.read(cx).value(), self.min, self.max, self.step);
        if (value - self.value).abs() > f64::EPSILON {
            self.value = value;
            cx.emit(SliderFieldEvent::Changed(value));
        }
        cx.notify();
    }

    fn on_input_changed(&mut self, value: f64, cx: &mut Context<Self>) {
        let value = normalize(value, self.min, self.max, self.step);
        if (value - self.value).abs() > f64::EPSILON {
            self.value = value;
            cx.emit(SliderFieldEvent::Changed(value));
        }
        cx.notify();
    }

    fn set_from_pointer(&mut self, value: f64, cx: &mut Context<Self>) {
        let value = normalize(value, self.min, self.max, self.step);
        if (value - self.value).abs() <= f64::EPSILON {
            return;
        }
        self.value = value;
        self.input
            .update(cx, |input, cx| input.set_value(value, cx));
        cx.emit(SliderFieldEvent::Changed(value));
        cx.notify();
    }
}

fn normalize(value: f64, min: f64, max: f64, step: f64) -> f64 {
    let value = value.clamp(min, max);
    let value = if step > 0. {
        (value / step).round() * step
    } else {
        value
    };
    (value * 1_000_000.).round() / 1_000_000.
}

impl Render for SliderField {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let percent = if self.max > self.min {
            ((self.value - self.min) / (self.max - self.min)).clamp(0., 1.) as f32
        } else {
            0.
        };
        let entity = cx.entity();
        let min = self.min;
        let max = self.max;
        let step = self.step;

        let value_from_x = move |x: Pixels, left: Pixels, width: Pixels| {
            let ratio = if width > px(0.) {
                ((x - left) / width).clamp(0., 1.)
            } else {
                0.
            };
            let raw = min + (max - min) * f64::from(ratio);
            if step > 0. {
                ((raw / step).round() * step).clamp(min, max)
            } else {
                raw
            }
        };

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(TEXT))
                                    .child(self.label.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(FAINT))
                                    .child(self.hint.clone()),
                            ),
                    )
                    .child(self.input.clone()),
            )
            .child(
                div()
                    .relative()
                    .h(px(24.))
                    .w_full()
                    .child(
                        div()
                            .absolute()
                            .top(px(10.))
                            .left_0()
                            .right_0()
                            .h(px(4.))
                            .rounded_full()
                            .bg(rgb(BORDER)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(10.))
                            .left_0()
                            .w(relative(percent))
                            .h(px(4.))
                            .rounded_full()
                            .bg(rgb(ACCENT)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(5.))
                            .left(relative(percent))
                            .ml(px(-7.))
                            .size(px(14.))
                            .rounded_full()
                            .border_2()
                            .border_color(rgb(BG))
                            .bg(rgb(ACCENT)),
                    )
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _cx| {
                                let left = bounds.origin.x;
                                let width = bounds.size.width;

                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |event: &MouseDownEvent, _, window, cx| {
                                        if !bounds.contains(&event.position) {
                                            return;
                                        }
                                        let value = value_from_x(event.position.x, left, width);
                                        entity.update(cx, |this, cx| {
                                            this.input
                                                .update(cx, |input, cx| input.commit(window, cx));
                                            this.set_from_pointer(value, cx);
                                            this.dragging = true;
                                        });
                                    }
                                });

                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |event: &MouseMoveEvent, _, _, cx| {
                                        let dragging = entity.read(cx).dragging;
                                        if !event.dragging() {
                                            if dragging {
                                                entity.update(cx, |this, _| this.dragging = false);
                                            }
                                            return;
                                        }
                                        if !dragging {
                                            return;
                                        }
                                        let value = value_from_x(event.position.x, left, width);
                                        entity.update(cx, |this, cx| {
                                            this.set_from_pointer(value, cx)
                                        });
                                    }
                                });

                                window.on_mouse_event({
                                    let entity = entity.clone();
                                    move |_: &MouseUpEvent, _, _, cx| {
                                        if entity.read(cx).dragging {
                                            entity.update(cx, |this, _| this.dragging = false);
                                        }
                                    }
                                });
                            },
                        )
                        .absolute()
                        .inset_0(),
                    ),
            )
    }
}
