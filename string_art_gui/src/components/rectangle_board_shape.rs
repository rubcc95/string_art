use dioxus::prelude::*;
use string_art_gui_common::input::board_shape::*;

use crate::{
    components::*,
    models::{self, board_shape::rectangle::Mode},
};

#[derive(Props, Clone, PartialEq)]
pub struct Props {
    on_change: Callback<Rectangle>,
    data: models::board_shape::Rectangle,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ModeController;

impl selector::Controller for ModeController {
    type Iter = [Mode; 2];

    fn items(&self) -> Self::Iter {
        [Mode::Auto, Mode::Manual]
    }

    fn to_index(item: &<Self::Iter as IntoIterator>::Item) -> usize {
        match item {
            Mode::Auto => 0,
            Mode::Manual => 1,
        }
    }

    fn from_index(&self, index: usize) -> <Self::Iter as IntoIterator>::Item {
        match index {
            0 => Mode::Auto,
            1 => Mode::Manual,
            other => panic!("Error parsing TableShape. Invalid index {}", other),
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
                Mode::Auto => "Auto",
                Mode::Manual => "Manual",
            }
        )
    }
}

#[component]
pub fn RectangleBoardShape(props: Props) -> Element {
    let res = match props.data.mode {
        Mode::Auto => {
            rsx! {
                Slider{
                    label: "Nail count:",
                    init_val: *props.data.auto,
                    range: 1..1024,
                    force_clamp: false,
                    step: 1,
                    on_change: move |val| {
                        props.on_change.call(Rectangle::Auto(rectangle::Auto(val)));
                    },
                }
            }
        }
        Mode::Manual => {
            rsx! {
                Slider{
                    label: "Width:",
                    init_val: props.data.manual.width,
                    range: 1..512,
                    step: 1,
                    on_change: move |width| {
                        props.on_change.call(Rectangle::Manual(rectangle::Manual{
                            width,
                            height: props.data.manual.height
                        }));
                    },
                },
                Slider{
                    label: "Height:",
                    init_val: props.data.manual.height,
                    range: 1..512,
                    step: 1,
                    on_change: move |height| {
                        props.on_change.call(Rectangle::Manual(rectangle::Manual{
                            width: props.data.manual.width,
                            height
                        }));
                    },
                }
            }
        }
    };

    rsx! {
        Flex{
            Selector{
                controller: ModeController,
                selection: props.data.mode,
                on_change: move |value| {
                    props.on_change.call(match value{
                        Mode::Auto => Rectangle::Auto(props.data.auto),
                        Mode::Manual => Rectangle::Manual(props.data.manual),
                    });
                },
            },
            {res}
        }
    }
}
