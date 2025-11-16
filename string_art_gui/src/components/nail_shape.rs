use super::*;
type Mode = models::nail_shape::Mode;

#[derive(Clone, Props, PartialEq)]
pub struct Props {
    data: Signal<models::NailShape>,
}

#[derive(Clone, Copy, PartialEq)]
struct ModeController;

impl selector::Controller for ModeController {
    type Iter = [Mode; 3];

    fn items(&self) -> Self::Iter {
        [Mode::Circular, Mode::Point, Mode::Hook]
    }

    fn to_index(item: &<Self::Iter as IntoIterator>::Item) -> usize {
        match item {
            Mode::Circular => 0,
            Mode::Point => 1,
            Mode::Hook => 2,
        }
    }

    fn from_index(&self, str: usize) -> <Self::Iter as IntoIterator>::Item {
        match str {
            0 => Mode::Circular,
            1 => Mode::Point,
            2 => Mode::Hook,
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
                Mode::Circular => "Circular",
                Mode::Point => "Point",
                Mode::Hook => "Hook",
            }
        )
    }
}

#[component]
pub fn NailShape(mut props: Props) -> Element {
    let state = (props.data)();
    let slider = match state.mode {
        Mode::Point => rsx!{},
        Mode::Circular => rsx! {
            Slider{
                label: "Radius",
                range: 0.0..10.0,
                step: 0.078125,
                init_val: state.circular,
                on_change: move |circular|{
                    props.data.write().circular = circular;
                    //props.on_change.call(input::NailShape::Circular(circular));
                },
            }
        },
        Mode::Hook => rsx! {
            Slider{
                label: "Width",
                range: 0.0..10.0,
                step: 0.078125,
                init_val: state.hook,
                on_change: move |hook|{
                    props.data.write().hook = hook;
                },
            }
        },
    };
    rsx! {
        FormGroup{
            legend: "Nail Shape",
            Flex{
                Selector{
                    controller: ModeController,
                    selection: state.mode,
                    on_change: move |mode|{
                        props.data.write().mode = mode;
                    },
                }
                {slider}
            }
        }
    }
}
