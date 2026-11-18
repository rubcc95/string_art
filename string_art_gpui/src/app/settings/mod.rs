//! Per-image settings panel and its model.

mod model;

pub(super) use model::{SettingField, Settings};

use gpui::{
    Context, Entity, EventEmitter, FocusHandle, Render, Subscription, Window, div, prelude::*,
};

use crate::ui::{SliderField, SliderFieldEvent};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum SettingsPanelEvent {
    Changed(Settings),
}

pub(super) struct SettingsPanel {
    settings: Settings,
    fields: Vec<(SettingField, Entity<SliderField>)>,
    /// Focus handle of the workspace's start button. It follows the numeric
    /// fields in the tab order and is owned here so the button can be rendered
    /// outside the panel.
    start_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SettingsPanelEvent> for SettingsPanel {}

impl SettingsPanel {
    pub fn new(settings: Settings, cx: &mut Context<Self>) -> Self {
        let mut fields = Vec::new();
        let mut subscriptions = Vec::new();

        for field in SettingField::ALL {
            let (min, max) = field.range(&settings);
            let slider = cx.new(|cx| {
                SliderField::new(
                    field.label(),
                    field.hint(),
                    field.suffix(),
                    field.value(&settings),
                    min,
                    max,
                    field.step(),
                    field.decimals(),
                    cx,
                )
            });
            let subscription = cx.subscribe(&slider, move |this, _, event, cx| {
                let SliderFieldEvent::Changed(value) = *event;
                this.on_field_changed(field, value, cx);
            });
            fields.push((field, slider));
            subscriptions.push(subscription);
        }

        Self {
            settings,
            fields,
            // Tab stops without an explicit index default to 0, so index 1
            // places the button after every numeric field.
            start_focus: cx.focus_handle().tab_stop(true).tab_index(1),
            _subscriptions: subscriptions,
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn start_focus(&self) -> FocusHandle {
        self.start_focus.clone()
    }

    #[allow(dead_code)]
    pub fn set_settings(&mut self, settings: Settings, cx: &mut Context<Self>) {
        self.settings = settings;
        for (field, slider) in &self.fields {
            let (min, max) = field.range(&settings);
            let value = field.value(&settings);
            slider.update(cx, |slider, cx| {
                slider.set_range(min, max, cx);
                slider.set_value(value, cx);
            });
        }
        cx.notify();
    }

    pub fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for (_, slider) in &self.fields {
            slider.update(cx, |slider, cx| slider.commit(window, cx));
        }
        let values = self
            .fields
            .iter()
            .map(|(field, slider)| (*field, slider.read(cx).value()))
            .collect::<Vec<_>>();
        for (field, value) in values {
            self.settings.apply(field, value);
        }
        cx.emit(SettingsPanelEvent::Changed(self.settings));
    }

    fn on_field_changed(&mut self, field: SettingField, value: f64, cx: &mut Context<Self>) {
        self.settings.apply(field, value);
        let applied = field.value(&self.settings);
        if (applied - value).abs() > f64::EPSILON {
            if let Some((_, slider)) = self
                .fields
                .iter()
                .find(|(candidate, _)| *candidate == field)
            {
                slider.update(cx, |slider, cx| slider.set_value(applied, cx));
            }
        }

        if field == SettingField::Nails {
            let (min, max) = SettingField::MinDistance.range(&self.settings);
            let value = self.settings.min_nail_distance as f64;
            if let Some((_, slider)) = self
                .fields
                .iter()
                .find(|(candidate, _)| *candidate == SettingField::MinDistance)
            {
                slider.update(cx, |slider, cx| {
                    slider.set_range(min, max, cx);
                    slider.set_value(value, cx);
                });
            }
        }

        cx.emit(SettingsPanelEvent::Changed(self.settings));
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_5()
            .children(self.fields.iter().map(|(_, slider)| slider.clone()))
    }
}
