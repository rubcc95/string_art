use derive_more::*;
use super::*;

#[derive(Clone, Props, PartialEq, Deref, DerefMut, Default)]
pub struct Props {
    #[deref]
    #[deref_mut]
    tab: models::NamedTab,
    on_build: Callback<(), SyncSignal<models::ComputationProcess>>,
}

#[component]
pub fn TabView(props: Props) -> Element {
    rsx! {
        div {
            BoardShape{
                data: props.tab.board_shape,
            },            
            NailShape{
                data: props.tab.nail_shape,
            },
            ComputationSettings{
                data: props.tab.computation_settings,
            },
            PipelineSettings{
                data: props.tab.pipeline_settings,
            },
            OutputGroup{
                tab: props.tab,
                on_build: props.on_build,
            }
        }
    }
}
