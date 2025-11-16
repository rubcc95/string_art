use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct FlexProps {
    children: Element,
}

#[component]
pub fn Flex(props: FlexProps) -> Element {
    rsx! {
        div{
            class: "flex",
            {props.children},
        }
    } 
}
