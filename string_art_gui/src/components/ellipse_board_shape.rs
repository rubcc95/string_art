use crate::components::{Flex, Slider};
use dioxus::prelude::*;
use string_art_gui_common::input::board_shape::Ellipse;

#[derive(Props, Clone, PartialEq)]
pub struct Props {
    data: Ellipse,
    on_change: Callback<Ellipse>,
}

#[component]
pub fn EllipseBoardShape(props: Props) -> Element {    
    rsx! {
        Flex{
            Slider {
                label: "Nail count:",
                range: 1..1024,
                init_val: props.data.nail_count,
                step: 1,
                on_change: move |val| {                    
                    props.on_change.call(Ellipse{
                        nail_count: val,
                        ..props.data
                    });
                }
            }
        },
        Flex{
            Slider {
                label: "Min nail distance count:",
                range: 0..(props.data.nail_count / 2).saturating_sub(1),
                init_val: props.data.min_nail_distance,
                step: 1,
                on_change: move |val|{
                    props.on_change.call(Ellipse{
                        nail_count: props.data.nail_count,
                        min_nail_distance: val,
                    });
                }
            }
        },
    }
}
