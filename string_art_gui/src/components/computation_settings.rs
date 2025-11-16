use super::*;
use models::computation_settings::DecayFnMode;
use string_art_gui_common::input::computation_settings::Precision;

#[derive(Clone, Props, PartialEq)]
pub struct Props {
    data: Signal<models::ComputationSettings>,
}

#[derive(Clone, Copy, PartialEq)]
struct DecayFnModeController;

impl selector::Controller for DecayFnModeController {
    type Iter = [DecayFnMode; 2];

    fn items(&self) -> Self::Iter {
        [DecayFnMode::Flat, DecayFnMode::Percentage]
    }

    fn to_index(item: &<Self::Iter as IntoIterator>::Item) -> usize {
        match item {
            DecayFnMode::Flat => 0,
            DecayFnMode::Percentage => 1,
        }
    }

    fn from_index(&self, str: usize) -> <Self::Iter as IntoIterator>::Item {
        match str {
            0 => DecayFnMode::Flat,
            1 => DecayFnMode::Percentage,
            other => panic!("Error parsing NailShape. Invalid index {}", other),
        }
    }

    fn display(
        &self,
        item: &<Self::Iter as IntoIterator>::Item,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match item {
                DecayFnMode::Flat => "Flat",
                DecayFnMode::Percentage => "Percentage",
            }
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct PrecisionController;

impl selector::Controller for PrecisionController {
    type Iter = [Precision; 4];

    fn items(&self) -> Self::Iter {
        [
            Precision::P8,
            Precision::P16,
            Precision::P32,
            Precision::P64,
        ]
    }

    fn to_index(item: &<Self::Iter as IntoIterator>::Item) -> usize {
        match item {
            Precision::P8 => 0,
            Precision::P16 => 1,
            Precision::P32 => 2,
            Precision::P64 => 3,
        }
    }

    fn from_index(&self, str: usize) -> <Self::Iter as IntoIterator>::Item {
        match str {
            0 => Precision::P8,
            1 => Precision::P16,
            2 => Precision::P32,
            3 => Precision::P64,
            other => panic!("Error parsing NailShape. Invalid index {}", other),
        }
    }

    fn display(
        &self,
        item: &<Self::Iter as IntoIterator>::Item,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match item {
                Precision::P8 => "8-bit",
                Precision::P16 => "16-bit",
                Precision::P32 => "32-bit",
                Precision::P64 => "64-bit",
            }
        )
    }
}

#[component]
pub fn ComputationSettings(mut props: Props) -> Element {
    let settings = (props.data)();
    let fn_rsx = match settings.decay_mode {
        DecayFnMode::Flat => rsx! {
            Slider{
                range: 0.0..1.0,
                step: 0.0078125,
                init_val: settings.decay_flat,
                on_change: move |val| {
                    props.data.write().decay_flat = val;
                },
            }
        },
        DecayFnMode::Percentage => rsx! {
            Slider{
                range: 0.0..1.0,
                step: 0.0078125,
                init_val: settings.decay_per,
                on_change: move |val| {
                    props.data.write().decay_per = val;
                },
            }
        },
    };
    rsx! {
        FormGroup{
            legend: "Computation Settings",
            Flex{
                Selector{
                    controller: DecayFnModeController,
                    selection: settings.decay_mode,
                    on_change: move |val| {
                        props.data.write().decay_mode = val;
                    },
                },
                {fn_rsx},
            },
            Flex{
                Slider{
                    label: "Resolution",
                    range: 128..2048,
                    step: 1,
                    init_val: settings.resolution,
                    on_change: move |val| {
                        props.data.write().resolution = val;
                    },
                },
                label { "Precision" },
                Selector{
                    controller: PrecisionController,
                    selection: settings.precision,
                    on_change: move |val| {
                        props.data.write().precision = val;
                    },
                }
            },
        }
    }
}
