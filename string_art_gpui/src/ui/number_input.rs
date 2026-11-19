//! Numeric text input widget.
//!
//! GPUI ships the text-input machinery but no widgets, so this adapts the
//! official `input` example into a reusable component: one entity per field,
//! restricted to digits and (when configured) a decimal point.

use std::ops::Range;
use std::sync::Arc;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable, FontFeatures, GlobalElementId,
    InspectorElementId, KeyDownEvent, LayoutId, PaintQuad, Pixels, Point, ShapedLine, SharedString,
    Style, Subscription, TextAlign, TextRun, UTF16Selection, UnderlineStyle, Window, div, fill,
    point, prelude::*, px, relative, rgb, rgba, size,
};

use super::{ACCENT, BG, BORDER, FAINT, TEXT};

/// OpenType feature list that keeps digits on a fixed advance.
fn tabular_figures() -> FontFeatures {
    FontFeatures(Arc::new(vec![("tnum".to_owned(), 1)]))
}

#[derive(Clone, Debug)]
pub struct NumberInputOptions {
    pub min: f64,
    pub max: f64,
    pub decimals: u8,
    pub suffix: SharedString,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumberInputEvent {
    Changed(f64),
}

pub struct NumberInput {
    focus_handle: FocusHandle,
    value: f64,
    text: String,
    editing: bool,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    min: f64,
    max: f64,
    decimals: u8,
    suffix: SharedString,
    blur_subscription: Option<Subscription>,
    focus_subscription: Option<Subscription>,
}

impl EventEmitter<NumberInputEvent> for NumberInput {}

impl NumberInput {
    pub fn new(value: f64, options: NumberInputOptions, cx: &mut Context<Self>) -> Self {
        let value = normalize(value, options.min, options.max, options.decimals);
        Self {
            focus_handle: cx.focus_handle().tab_stop(true),
            value,
            text: format_value(value, options.decimals),
            editing: false,
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            min: options.min,
            max: options.max,
            decimals: options.decimals,
            suffix: options.suffix,
            blur_subscription: None,
            focus_subscription: None,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn set_value(&mut self, value: f64, cx: &mut Context<Self>) {
        self.value = normalize(value, self.min, self.max, self.decimals);
        if !self.editing {
            self.text = format_value(self.value, self.decimals);
        }
        cx.notify();
    }

    pub fn set_range(&mut self, min: f64, max: f64, cx: &mut Context<Self>) {
        self.min = min;
        self.max = max;
        self.value = normalize(self.value, self.min, self.max, self.decimals);
        if !self.editing {
            self.text = format_value(self.value, self.decimals);
        }
        cx.notify();
    }

    /// Parses the draft and emits a change if the committed value differs.
    pub fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.editing {
            return;
        }
        let previous = self.value;
        if let Ok(value) = self.text.trim().parse::<f64>() {
            self.value = normalize(value, self.min, self.max, self.decimals);
        }
        self.text = format_value(self.value, self.decimals);
        self.editing = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        window.notify_text_input_changed(&self.focus_handle);
        if (self.value - previous).abs() > f64::EPSILON {
            cx.emit(NumberInputEvent::Changed(self.value));
        }
        cx.notify();
    }

    pub fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.editing {
            return;
        }
        self.text = format_value(self.value, self.decimals);
        self.editing = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    fn begin_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.blur_subscription.is_none() {
            self.blur_subscription =
                Some(cx.on_blur(&self.focus_handle, window, |this, window, cx| {
                    this.commit(window, cx);
                    // Focus listeners run at the end of the frame, when
                    // `notify` is ignored. Deferring it guarantees a repaint
                    // without waiting for the next mouse event.
                    cx.defer_in(window, |_, _, cx| cx.notify());
                }));
        }
        if self.editing {
            window.focus(&self.focus_handle, cx);
            window.request_virtual_keyboard();
            cx.notify();
            return;
        }
        self.text = format_value(self.value, self.decimals);
        self.selected_range = 0..self.text.len();
        self.selection_reversed = false;
        self.marked_range = None;
        self.editing = true;
        window.focus(&self.focus_handle, cx);
        window.request_virtual_keyboard();
        cx.notify();
    }

    fn handle_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;

        // Tab navigates between fields even when none of them is being edited,
        // so it is handled before the editing guard. Committing here keeps the
        // typed value before the focus moves to the next control.
        if key == "tab" {
            self.commit(window, cx);
            if modifiers.shift {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            cx.stop_propagation();
            return;
        }

        if !self.editing {
            return;
        }
        let shortcut = modifiers.platform || modifiers.control;
        let cursor = self.cursor_offset();
        let text_len = self.text.len();
        let selected_range = self.selected_range.clone();
        let selected_text = self.text[selected_range.clone()].to_string();

        match key {
            "enter" => {
                self.commit(window, cx);
                cx.stop_propagation();
            }
            "escape" => {
                self.cancel(window, cx);
                cx.stop_propagation();
            }
            "backspace" => {
                self.backspace(window, cx);
                cx.stop_propagation();
            }
            "delete" => {
                self.delete(window, cx);
                cx.stop_propagation();
            }
            "left" => {
                let offset = cursor.saturating_sub(1);
                if modifiers.shift {
                    self.select_to(offset, window, cx);
                } else {
                    self.move_to(offset, window, cx);
                }
                cx.stop_propagation();
            }
            "right" => {
                let offset = (cursor + 1).min(text_len);
                if modifiers.shift {
                    self.select_to(offset, window, cx);
                } else {
                    self.move_to(offset, window, cx);
                }
                cx.stop_propagation();
            }
            "home" => {
                self.move_to(0, window, cx);
                cx.stop_propagation();
            }
            "end" => {
                self.move_to(text_len, window, cx);
                cx.stop_propagation();
            }
            "a" if shortcut => {
                self.select_all(window, cx);
                cx.stop_propagation();
            }
            "v" if shortcut => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.insert_text(&text, window, cx);
                }
                cx.stop_propagation();
            }
            "c" if shortcut => {
                if !selected_range.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
                }
                cx.stop_propagation();
            }
            "x" if shortcut => {
                if !selected_range.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(selected_text));
                    self.insert_text("", window, cx);
                }
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn move_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    fn select_all(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.selected_range = 0..self.text.len();
        self.selection_reversed = false;
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    fn backspace(&mut self, window: &Window, cx: &mut Context<Self>) {
        let range = if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            cursor.saturating_sub(1)..cursor
        } else {
            self.selected_range.clone()
        };
        if range.start != range.end {
            self.text.replace_range(range.clone(), "");
            self.selected_range = range.start..range.start;
            self.selection_reversed = false;
            window.notify_text_input_changed(&self.focus_handle);
            cx.notify();
        }
    }

    fn delete(&mut self, window: &Window, cx: &mut Context<Self>) {
        let range = if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            cursor..(cursor + 1).min(self.text.len())
        } else {
            self.selected_range.clone()
        };
        if range.start != range.end {
            self.text.replace_range(range.clone(), "");
            self.selected_range = range.start..range.start;
            self.selection_reversed = false;
            window.notify_text_input_changed(&self.focus_handle);
            cx.notify();
        }
    }

    fn insert_text(&mut self, text: &str, window: &Window, cx: &mut Context<Self>) {
        let range = self.selected_range.clone();
        self.replace_range(range, text, window, cx);
    }

    fn replace_range(
        &mut self,
        range: Range<usize>,
        text: &str,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let before = self.text[..range.start].to_string();
        let after = self.text[range.end..].to_string();
        let mut insert = sanitize_text(text, self.decimals > 0);
        if self.decimals > 0
            && insert.contains('.')
            && (before.contains('.') || after.contains('.'))
        {
            insert = insert.replace('.', "");
        }

        self.text = format!("{before}{insert}{after}");
        let cursor = before.len() + insert.len();
        self.selected_range = cursor..cursor;
        self.selection_reversed = false;
        self.marked_range = None;
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        let start = self.offset_from_utf16(range_utf16.start);
        start..self.offset_from_utf16(range_utf16.end)
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.text.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }
}

fn normalize(value: f64, min: f64, max: f64, decimals: u8) -> f64 {
    let value = value.clamp(min, max);
    let factor = 10f64.powi(i32::from(decimals));
    (value * factor).round() / factor
}

fn format_value(value: f64, decimals: u8) -> String {
    if decimals == 0 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.prec$}", prec = usize::from(decimals))
    }
}

fn sanitize_text(text: &str, allow_decimal: bool) -> String {
    let mut out = String::new();
    let mut has_dot = false;
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            out.push(ch);
        } else if ch == '.' && allow_decimal && !has_dot {
            has_dot = true;
            out.push('.');
        }
        if out.len() >= 12 {
            break;
        }
    }
    out
}

impl EntityInputHandler for NumberInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        self.text.get(range).map(|text| text.to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn text_length_utf16(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.text.encode_utf16().count())
    }

    fn accepts_text_input(&self, _window: &mut Window, _cx: &mut Context<Self>) -> bool {
        self.editing
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_range = self.range_from_utf16(&range_utf16);
        self.selection_reversed = false;
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());
        self.replace_range(range, new_text, window, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_text_in_range(range_utf16, new_text, window, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        if line.text != self.text {
            return None;
        }
        let range = self.range_from_utf16(&range_utf16);
        let x_offset = bounds.size.width - line.width();
        Some(Bounds::from_corners(
            point(
                bounds.left() + x_offset + line.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + x_offset + line.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn selection_bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Bounds<Pixels>> {
        self.bounds_for_range(range_utf16, bounds, window, cx)
            .into_iter()
            .collect()
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let local = self.last_bounds?.localize(&point)?;
        let line = self.last_layout.as_ref()?;
        if line.text != self.text {
            return None;
        }
        let x_offset = self.last_bounds?.size.width - line.width();
        Some(line.closest_index_for_x(local.x - x_offset))
    }
}

impl Focusable for NumberInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NumberInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Keyboard navigation moves focus into this field without a click, so
        // entering edit mode cannot depend on `begin_edit`'s mouse handler.
        // The listener needs a window, hence it is registered on first render.
        if self.focus_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_subscription =
                Some(cx.on_focus(&focus_handle, window, |this, window, cx| {
                    this.begin_edit(window, cx);
                    // See the blur listener: a notify during the focus phase is
                    // dropped, so schedule it for after the frame.
                    cx.defer_in(window, |_, _, cx| cx.notify());
                }));
        }

        let editing = self.editing;
        let focused = self.focus_handle.is_focused(window);
        let suffix = self.suffix.clone();
        let value = format_value(self.value, self.decimals);

        div()
            .h(px(34.))
            .w(px(96.))
            .flex_none()
            .flex()
            .items_center()
            .justify_end()
            .gap_1()
            .px_2()
            .rounded_md()
            .border_1()
            .border_color(rgb(if focused && editing { ACCENT } else { BORDER }))
            .bg(rgb(BG))
            .text_sm()
            .text_color(rgb(TEXT))
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| this.begin_edit(window, cx)),
            )
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event, window, cx| {
                this.handle_key(event, window, cx);
            }))
            .child(if editing {
                NumberInputElement::new(cx.entity()).into_any_element()
            } else {
                div()
                    .text_sm()
                    .text_color(rgb(TEXT))
                    .font_features(tabular_figures())
                    .child(value)
                    .into_any_element()
            })
            .when(!suffix.is_empty(), |number_box| {
                number_box.child(div().text_xs().text_color(rgb(FAINT)).child(suffix))
            })
    }
}

/// The painted text of the edited field. It registers the input handler so the
/// platform, including the mobile virtual keyboard, can edit it.
struct NumberInputElement {
    input: Entity<NumberInput>,
}

impl NumberInputElement {
    fn new(input: Entity<NumberInput>) -> Self {
        Self { input }
    }
}

pub(super) struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl Element for NumberInputElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        if !input.editing {
            return PrepaintState {
                line: None,
                cursor: None,
                selection: None,
            };
        }

        let content = input.text.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();
        let mut font = style.font();
        // Tabular figures: the value must not shift while it is being edited.
        font.features = tabular_figures();

        let run = TextRun {
            len: content.len(),
            font,
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: content.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(content.into(), font_size, &runs, None);

        let x_offset = bounds.size.width - line.width();
        let cursor_pos = x_offset + line.x_for_index(cursor);
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(2.), bounds.bottom() - bounds.top()),
                    ),
                    gpui::blue(),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + x_offset + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + x_offset + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    rgba(0x2563eb40),
                )),
                None,
            )
        };

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );

        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let Some(line) = prepaint.line.take() else {
            return;
        };
        line.paint(
            bounds.origin,
            window.line_height(),
            TextAlign::Right,
            Some(bounds.size.width),
            window,
            cx,
        )
        .ok();

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}
