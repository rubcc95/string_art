use super::*;
use models::pipeline_settings::Mode;

#[derive(Clone, Props, PartialEq)]
pub struct Props {
    data: Signal<models::PipelineSettings>,
}

#[derive(Clone, Copy, PartialEq)]
struct ModeController;

impl selector::Controller for ModeController {
    type Iter = [Mode; 2];

    fn items(&self) -> Self::Iter {
        [Mode::Mono, Mode::Multi]
    }

    fn to_index(item: &<Self::Iter as IntoIterator>::Item) -> usize {
        match item {
            Mode::Mono => 0,
            Mode::Multi => 1,
        }
    }

    fn from_index(&self, str: usize) -> <Self::Iter as IntoIterator>::Item {
        match str {
            0 => Mode::Mono,
            1 => Mode::Multi,
            other => panic!("Error parsing pipeline mode. Invalid index {}", other),
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
                Mode::Mono => "Monochrome",
                Mode::Multi => "Multicolor",
            }
        )
    }
}
#[component] 
pub fn PipelineSettings(mut props: Props) -> Element {
    let data = (props.data)();
    rsx! {
        FormGroup{
            legend: "Nail Shape",
            Flex{
                Selector{
                    controller: ModeController,
                    selection: data.mode,
                    on_change: move |mode|{
                        props.data.write().mode = mode;
                    },
                },
                if data.mode == Mode::Mono{
                    Slider{
                        label: "Thread Count",
                        range: 1..20000,
                        step: 1,
                        init_val: data.mono,
                        on_change: move |threads|{
                            props.data.write().mono = threads;
                        },
                    }
                }
            }
        }
    }
}
