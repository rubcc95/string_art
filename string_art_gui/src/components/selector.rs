use dioxus::prelude::*;

#[allow(type_alias_bounds)]
type Item<S: Controller> = <S::Iter as IntoIterator>::Item;

#[derive(Props, Clone, PartialEq)]
pub struct SelectorProps<V: Controller> {
    controller: V,
    selection: Item<V>,
    #[props(default)]
    on_change: Callback<Item<V>>,
}

pub trait Controller: Clone + PartialEq + 'static {
    type Iter: IntoIterator<Item: Clone + PartialEq + 'static>;

    fn items(&self) -> Self::Iter;

    fn to_index(item: &<Self::Iter as IntoIterator>::Item) -> usize;

    fn from_index(&self, index: usize) -> <Self::Iter as IntoIterator>::Item;

    fn display(
        &self,
        item: &<Self::Iter as IntoIterator>::Item,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result;
}

#[component]
fn Test(index: usize) -> Element{
    let a = use_signal(Vec::<Signal<String>>::new);
    let b = use_memo(move || {
        let signal = a.read()[index];
        signal()
    });
    
    rsx!{
        label{ "hi im {b()}"}
    }
}

struct SelectorDisplayer<'a, S: Controller> {
    controller: &'a S,
    item: Item<S>,
}

impl<'a, S: Controller> SelectorDisplayer<'a, S> {
    pub fn new(controller: &'a S, item: Item<S>) -> Self {
        Self { controller, item }
    }
}

impl<S: Controller> std::fmt::Display for SelectorDisplayer<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.controller.display(&self.item, f)
    }
}

#[component]
pub fn Selector<V: Controller>(props: SelectorProps<V>) -> Element {
    let selection = V::to_index(&props.selection);
    rsx! {
            select {
                class: "padded",
                onchange: move |a| props.on_change.call(props.controller.from_index(a.parsed().unwrap())),
                for (index, displayer) in props.controller.items().into_iter().map(|element| (V::to_index(&element), SelectorDisplayer::new(&props.controller, element))) {
                    option {
                        value: index,
                        label: "{displayer}",
                        selected: index == selection,
                    }
                }
            }
    }
}
