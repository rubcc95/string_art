use derive_more::*;
use dioxus::prelude::*;
use std::rc::Rc;
use string_art_gui_common::input;

pub use board_shape::BoardShape;
pub use computation_settings::ComputationSettings;
pub use nail_shape::NailShape;
pub use pipeline_settings::PipelineSettings;

use crate::signals::SignalVec;

pub mod board_shape;
pub mod computation_settings;
pub mod nail_shape;
pub mod pipeline_settings;

pub type Image = input::Image;

#[derive(Clone, PartialEq, Default, Deref, DerefMut)]
pub struct NamedTab {
    pub name: Rc<str>,
    #[deref]
    #[deref_mut]
    pub tab: Tab,
}

impl NamedTab {
    pub fn new(name: Rc<str>, image: Signal<Option<Result<Image, String>>>) -> Self {
        Self {
            name,
            tab: Tab {
                image,
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default, Deref, DerefMut)]
pub struct Tab {
    pub image: Signal<Option<Result<Image, String>>>,
    #[deref]
    #[deref_mut]
    pub form_data: FormData,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct FormData {
    pub board_shape: Signal<BoardShape>,
    pub nail_shape: Signal<NailShape>,
    pub computation_settings: Signal<ComputationSettings>,
    pub pipeline_settings: Signal<PipelineSettings>,
    pub outputs: SignalVec<SyncSignal<ComputationProcess>>,
}

#[derive(Default)]
pub enum ComputationProcess {
    #[default]
    Waiting,
    Working(string_art_gui_common::Data, InstructionsProcess),
    Err(String),
}

pub enum InstructionsProcess {
    Waiting(Vec<string_art_gui_common::Step>),
    Done(String),
}

impl Default for InstructionsProcess {
    fn default() -> Self {
        Self::Waiting(Vec::new())
    }
}