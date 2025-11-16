use dioxus::prelude::*;
use std::{ops::Range, str::FromStr};

// #[derive(Props, PartialEq, Clone)]
// pub struct SliderProps<V: SliderValue> {
//     #[props(default = "")]
//     label: &'static str,
//     #[props(default)]
//     range: Range<V>,
//     #[props(default)]
//     handle: Signal<V>,
//     #[props(default)]
//     step: V,
//     #[props(default = true)]
//     force_clamp: bool,
//     #[props(default = false)]
//     disabled: bool,
// }

pub trait SliderValue:
    FromStr + IntoAttributeValue + Clone + PartialEq + PartialOrd + 'static
{
}

impl<T: FromStr + Clone + PartialEq + IntoAttributeValue + PartialOrd + 'static> SliderValue for T {}

// #[component]
// pub fn Slider<V: SliderValue>(mut props: SliderProps<V>) -> Element {
//     rsx! {
//         label { {props.label} },
//         input {
//             r#type: "range",
//             disabled: props.disabled,
//             class: "extent",
//             min: props.range.start.clone(),
//             max: props.range.end.clone(),
//             step: props.step.clone(),
//             onchange: move |e|{
//                 if let Ok(v) = e.parsed::<V>(){
//                     props.handle.set(v);
//                 }
//             },
//             value: props.handle,
//         }
//         input {
//             r#type: "number",
//             disabled: props.disabled,
//             class: "form-slider-number",
//             value: props.handle,
//             step: props.step,
//             onchange: move |e|{
//                 if let Ok(mut v) = e.parsed::<V>(){
//                     if props.force_clamp{
//                         if v < props.range.start{
//                             v = props.range.start.clone();
//                         } else if v > props.range.end{
//                             v = props.range.end.clone();
//                         }
//                     }
//                     props.handle.set(v);
//                 }
//             }
//         }
//     }
// }

#[derive(Props, PartialEq, Clone)]
pub struct SliderProps<V: SliderValue> {
    #[props(default = "")]
    label: &'static str,
    range: Range<V>,
    init_val: V,
    step: V,
    #[props(default = true)]
    force_clamp: bool,
    #[props(default = false)]
    disabled: bool,
    #[props(default)]
    on_change: Callback<V>,
}

#[component]
pub fn Slider<V: SliderValue>(props: SliderProps<V>) -> Element {
    rsx! {
        label {
             {props.label}
        },
        input {
            r#type: "range",
            disabled: props.disabled,
            class: "extent",
            min: props.range.start.clone(),
            max: props.range.end.clone(),
            step: props.step.clone(),
            onchange: move |e|{
                if let Ok(v) = e.parsed::<V>(){
                    props.on_change.call(v);
                }
            },
            value: props.init_val.clone(),
        }
        input {
            r#type: "number",
            disabled: props.disabled,
            class: "padded small-input",
            value: props.init_val,
            step: props.step,
            onchange: move |e|{
                if let Ok(mut v) = e.parsed::<V>(){
                    if props.force_clamp{
                        if v < props.range.start{
                            v = props.range.start.clone();
                        } else if v > props.range.end{
                            v = props.range.end.clone();
                        }
                    }
                    props.on_change.call(v);
                }
            }
        }
    }
}
