#![windows_subsystem = "windows"]
pub mod components;
pub mod models;
pub mod signals;
use dioxus::{html::FileEngine, prelude::*};
use signals::*;
use std::sync::Arc;

fn main() {
    #[cfg(feature = "desktop")]
    {
        LaunchBuilder::new()
            .with_cfg(
                dioxus::desktop::Config::new()
                    .with_menu(None)
                    .with_disable_drag_drop_handler(false),
            )
            .launch(App);
    }
    #[cfg(not(feature = "desktop"))]
    {
        dioxus::launch(App);
    }
}

const STYLES: Asset = asset!("/assets/styles.css");

#[component]
fn App() -> Element {
    let mut props = use_signal_vec::<models::NamedTab>();

    let on_drop = move |file_engine: Arc<dyn FileEngine>| {
        for file in file_engine.files() {
            if let Ok(format) = image::ImageFormat::from_path(&file) {
                let file_engine = file_engine.clone();
                let mut image = use_signal(|| None);
                props.push(models::NamedTab::new(file.clone().into(), image));
                spawn(async move {
                    image.set(Some(match file_engine.read_file(&file).await {
                        Some(bytes) => match image::load_from_memory_with_format(&bytes, format) {
                            Ok(image) => Ok(image.to_rgb8().into()),
                            Err(err) => Err(err.to_string()),
                        },
                        None => Err("Image was not selected".to_string()),
                    }));
                });
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STYLES },
        components::DropZone{
            on_drop: on_drop,
            dragging: rsx!{
                div {
                    class: "drop-zone-full",
                    div {
                        class: "drop-box dragover",
                        "Drop files here",
                    },
                }
            },
            HomePage {
                tabs: props,
                on_file_added: on_drop,
                on_build: move |mut tab: models::NamedTab| {
                    let output = SyncSignal::default();
                    tab.form_data.outputs.push(output);
                    output
                }
            },
        },
    }
}

#[derive(Clone, Props, PartialEq)]
pub struct HomePageProps {
    tabs: SignalVec<models::NamedTab>,
    on_file_added: Callback<Arc<dyn FileEngine>>,
    on_build: Callback<models::NamedTab, SyncSignal<models::ComputationProcess>>,
}

#[component]
pub fn HomePage(mut props: HomePageProps) -> Element {
    //info!("Builded picker!");
    let curr_idx = props.tabs.selection_idx();
    let tabs = props.tabs.read();
    match tabs.len() == 0 {
        true => {
            rsx! {
                div {
                    class: "drop-zone-full",
                    div{
                        class: "drop-box",
                        label {
                            style: "cursor: pointer",
                            input {
                                r#type: "file",
                                accept: ".bmp,.dds,.ff,.gif,.hdr,.ico,.jpg,.jpeg,.exr,.png,.pbm,.pgm,.ppm,.pam,.qoi,.tga,.tiff,.tif,.webp",
                                multiple: true,
                                name: "textreader",
                                directory: false,
                                style: "display: none;",
                                oninput: move |evt: FormEvent| {
                                    if let Some(files) = evt.files() {
                                        spawn({
                                            async move {
                                                props.on_file_added.call(files);
                                            }
                                        });
                                    }
                                }
                            },
                            "Click or drag files here"
                        }
                    },
                }
            }
        }
        false => {
            let iter = tabs.iter().enumerate().map(|(id, tab)| {
                rsx! {
                    div {
                        class: if Some(id) == curr_idx { "tab active" } else { "tab" },
                        onclick: move |_| props.tabs.set_selection(Some(id)),
                        "{tab.name}",
                        button {
                            r#type: "button",
                            class: "close-btn",
                            onclick: move |e| {
                                e.stop_propagation();
                                props.tabs.remove(id);
                            },
                            "×"
                        }
                    }
                }
            });
            rsx! {
                div {
                    class: "tabs",
                    {iter}
                },
                if let Some(active_tab) = props.tabs.selection(){
                    components::TabView{
                        tab: active_tab.clone(),
                        on_build: move |_| {
                            props.on_build.call(active_tab.clone())
                        },
                    }
                }
            }
        }
    }
}
