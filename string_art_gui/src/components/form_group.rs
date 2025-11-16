use dioxus::prelude::*; 

#[derive(Props, Clone, PartialEq)]
pub struct FormGroupProps {
    children: Element,
    legend: &'static str,
}

#[component]
pub fn FormGroup(props: FormGroupProps) -> Element {
    rsx! {
        fieldset {
            class: "form-group",
            legend { {props.legend} },
            {props.children}
        } 
    }
}
