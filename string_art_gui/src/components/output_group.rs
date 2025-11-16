use super::*;
use derive_more::*;
use string_art_gui_common as common;

#[derive(Clone, Props, PartialEq, Deref, DerefMut, Default)]
pub struct Props {
    #[deref]
    #[deref_mut]
    tab: models::NamedTab,
    on_build: Callback<(), SyncSignal<models::ComputationProcess>>,
}

#[component]
pub fn OutputGroup(props: Props) -> Element {
    let output = match &*props.image.read() {
        Some(Ok(_)) => {
            let mut outputs = props.outputs;
            let curr_idx = outputs.selection_idx();

            let tabs = (0..outputs.read().len()).map(|idx| {
                rsx! {
                    div {
                        class: if Some(idx) == curr_idx { "tab active" } else { "tab" },
                        onclick: move |_| outputs.set_selection(Some(idx)),
                        {(idx + 1).to_string()},
                        button {
                            r#type: "button",
                            class: "close-btn",
                            onclick: move |e| {
                                e.stop_propagation();
                                outputs.remove(idx);
                            },
                            "×"
                        }
                    }
                }
            });

            let state = props.form_data;
            let image = (props.image).unwrap().unwrap();

            let mut on_click = Some(move |_| {
                let output = props.on_build.call(());
                compute(
                    common::Input {
                        board_shape: (state.board_shape)().into(),
                        nail_shape: (state.nail_shape)().into(),
                        image,
                        computation_settings: (state.computation_settings)().into(),
                        pipeline: (state.pipeline_settings)().into(),
                    },
                    output,
                );
            });

            let compute_button = rsx! {
                button{
                    class: "tab",
                    r#type: "button",
                    onclick: move |evt| {
                        if let Some(onclick) = on_click.take(){
                            onclick(evt);
                        } else { }
                    },
                    "Start"
                }
            };

            let (progress_bar, svg_view) = if let Some(output) = props.tab.outputs.selection() {
                match &*output.read() {
                    models::ComputationProcess::Waiting => (rsx! { label{"Queueing..."} }, rsx! {}),
                    models::ComputationProcess::Working(data, instructions) => (
                        match instructions {
                            models::InstructionsProcess::Waiting(steps) => {
                                let progress =
                                    100.0 * steps.len() as f32 / data.estimated_steps as f32;
                                rsx! {
                                    div {
                                        class: "progress-container",
                                        div {
                                            class: "progress-bar",
                                            style: "width: {progress}%;",
                                        }
                                    }
                                }
                            }
                            models::InstructionsProcess::Done(instructions) => rsx! {
                                InstructionsToClipboard {
                                    instructions: instructions.clone().into(),
                                }
                            },
                        },
                        rsx! {
                            div { dangerous_inner_html: data.svg.to_string() },
                        },
                    ),
                    models::ComputationProcess::Err(err) => {
                        (rsx! {}, rsx! { label{ "Error: {err}"} })
                    }
                }
            } else {
                (rsx! {}, rsx! {})
            };

            rsx! {
                div {
                    class: "tabs",
                    {tabs},
                    {compute_button},
                    {progress_bar}
                },
                {svg_view}
            }
        }
        Some(Err(err)) => rsx! {
            "Failed loading image: {err}"
        },
        None => rsx! {"Loading, please wait"},
    };
    rsx! {
        FormGroup{
            legend: "Builds",
            {output}
        }
    }
}

#[component]
pub fn ProgressBar(progress: f32) -> Element {
    rsx! {
        div {
            class: "progress-container",
            div {
                class: "progress-bar",
                style: "width: {progress}%;",
            }
        }
    }
}

fn compute(input: common::Input, mut data: SyncSignal<models::ComputationProcess>) {
    let instructions_builder = input.instrucions_builder();

    let mut on_output = move |output| {
        let mut writer = data.write();
        match output {
            common::Output::Init(svg) => {
                *writer = models::ComputationProcess::Working(
                    svg,
                    models::InstructionsProcess::Waiting(Vec::new()),
                );
            }

            common::Output::Steps(new_steps) => {
                if let models::ComputationProcess::Working(
                    data,
                    models::InstructionsProcess::Waiting(steps),
                ) = &mut *writer
                {
                    if let Err(err) = data.append_steps(steps, new_steps) {
                        *writer = models::ComputationProcess::Err(err);
                    }
                } else {
                    unreachable!();
                }
            }

            common::Output::Done(new_steps, anchors) => {
                if let models::ComputationProcess::Working(data, instructions) = &mut *writer {
                    if let models::InstructionsProcess::Waiting(steps) = instructions {
                        if let Err(err) = data.append_steps(steps, new_steps) {
                            *writer = models::ComputationProcess::Err(err);
                            return;
                        }

                        let mut text = String::new();

                        for anchor in anchors {
                            if let Err(err) = instructions_builder(&mut text, &anchor) {
                                *writer = models::ComputationProcess::Err(err.to_string());
                                return;
                            }
                        }

                        for step in steps.iter().rev() {
                            if let Err(err) = instructions_builder(&mut text, &step.anchor) {
                                *writer = models::ComputationProcess::Err(err.to_string());
                                return;
                            }
                        }

                        *instructions = models::InstructionsProcess::Done(text);
                    } else {
                        unreachable!();
                    }
                } else {
                    unreachable!();
                }
            }

            common::Output::Err(err) => {
                *writer = models::ComputationProcess::Err(err);
            }
        }
    };

    #[cfg(feature = "native")]
    {
        rayon::spawn(move || {
            let mut computation = match common::computation(input) {
                Ok(computation) => computation,
                Err(err) => return on_output(common::Output::Err(err.to_string())),
            };

            match computation.init() {
                Ok(data) => {
                    on_output(common::Output::Init(data));

                    let mut to_send = Vec::new();

                    while let Some(output) = computation.next() {
                        match output {
                            Ok(step) => {
                                to_send.push(step);
                                if to_send.len() >= 32 {                                    on_output(common::Output::Steps(to_send));
                                    to_send = Vec::new();
                                }
                            }
                            Err(err) => return on_output(common::Output::Err(err.to_string())),
                        }
                    }

                    on_output(common::Output::Done(to_send, computation.start_anchors()));
                }
                Err(err) => return on_output(common::Output::Err(err.to_string())),
            }
        });
    }

    #[cfg(feature = "web")]
    {
        use futures::*;
        use string_art_gui_web_worker::WebWorker;

        const RAW_ASSET_FOLDER: Asset = asset!("/assets/pkg");

        spawn(async move {
            let mut worker = WebWorker::spawn(&format!(
                "{}/string_art_gui_web_worker_registrar.js",
                RAW_ASSET_FOLDER.to_string()
            ));
            worker.send(input).await.unwrap();
            while let Some(out) = worker.next().await {
                on_output(out);
            }
        });
    }
}
