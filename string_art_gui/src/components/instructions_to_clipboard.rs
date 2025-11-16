use super::*;
use std::rc::Rc;

#[derive(Clone, Props, PartialEq)]
pub struct Props {
    pub instructions: Rc<str>,
}

#[component]
pub fn InstructionsToClipboard(props: Props) -> Element {
    let code = props.instructions.clone();
    let on_copy = move |_| {
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.navigator().clipboard().write_text(&code);
            }
        }
        #[cfg(feature = "desktop")]
        {
            let mut clipboard = arboard::Clipboard::new().unwrap();
            clipboard.set_text(&*code).unwrap();
        }
    };

    rsx! {
            button {
                class: "aaaa",
                r#type: "button",
                onclick: on_copy,
                "Copiar"
        }
    }
}
