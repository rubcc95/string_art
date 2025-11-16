use dioxus::prelude::*;
use dioxus_elements::{FileEngine, HasFileData};
use std::sync::Arc;

#[derive(Clone, Props, PartialEq)]
pub struct DropZoneProps {
    on_drop: Callback<Arc<dyn FileEngine>>,
    dragging: Element,
    children: Element,
}

#[component]
pub fn DropZone(props: DropZoneProps) -> Element {
    let mut is_dragging = use_signal(|| false);

    rsx! {
        div {
            ondragenter: {
                #[cfg(feature = "web")]
                {
                    move |evt: DragEvent| {
                        is_dragging.set(true);
                        evt.prevent_default();
                    }
                }
                #[cfg(not(feature = "web"))]
                {
                    |_| {}
                }
            },
            ondragleave: move |evt| {
                is_dragging.set(false);
                evt.prevent_default();
            },
            ondrop: move |evt| {
                #[cfg(feature = "web")]
                {
                    is_dragging.set(false);
                }
                if let Some(files) =evt.files() {
                    props.on_drop.call(files);
                }
                evt.prevent_default();
            },
            ondragover: move |evt| {
                #[cfg(feature = "desktop")]
                {
                    is_dragging.set(true);
                }
                evt.prevent_default();
            },
            if is_dragging() {
                {props.dragging}
            } else{
                {props.children}
            }
        }
    }
}
