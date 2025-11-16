use super::*;
use models::board_shape::Mode;

#[derive(Clone, Props, PartialEq)]
pub struct BoardShapeProps {
    data: Signal<models::BoardShape>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ModeController;

impl selector::Controller for ModeController {
    type Iter = [Mode; 2];

    fn items(&self) -> Self::Iter {
        [Mode::Ellipse, Mode::Rectangle]
    }

    fn to_index(item: &<Self::Iter as IntoIterator>::Item) -> usize {
        match item {
            Mode::Ellipse => 0,
            Mode::Rectangle => 1,
        }
    }

    fn from_index(&self, index: usize) -> <Self::Iter as IntoIterator>::Item {
        match index {
            0 => Mode::Ellipse,
            1 => Mode::Rectangle,
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
                Mode::Ellipse => "Ellipse",
                Mode::Rectangle => "Rectangle",
            }
        )
    }
}

#[component]
pub fn BoardShape(mut props: BoardShapeProps) -> Element {
    let state = (props.data)();
    let res = match state.mode {
        Mode::Ellipse => rsx! {
            super::EllipseBoardShape{
                data: state.ellipse,
                on_change: move |ellipse| {
                    props.data.write().ellipse = ellipse;
                },
            }
        },
        Mode::Rectangle => rsx! {
            super::RectangleBoardShape{
                data: state.rectangle,
                on_change: move |rectangle| {
                    props.data.write().rectangle.set(rectangle);
                },
            }
        },
    };

    rsx! {
        FormGroup{
            legend: "Board Shape",
            Flex{
                Selector{
                    controller: ModeController,
                    selection: state.mode,
                    on_change: move |value| {
                        props.data.write().mode = value;
                    },
                },
            }
            {res}
        }
    }
}
