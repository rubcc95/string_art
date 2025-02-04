use std::{ops::Deref, sync::Arc};

use egui::mutex::Mutex;
use string_art::{
    color, nails, slice, verboser::{self, Verboser}, Baked, Computation as Cmp
};

use crate::{
    args::{LineConfigState, Args},
    SyncArgs,
};

#[derive(Default)]
pub struct Synced<T>(Arc<Mutex<T>>);

impl<T> Clone for Synced<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Synced<T> {
    type Target = Mutex<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait SyncedBuilder:
    nails::Builder<
    Handle: nails::Handle<Nail: Send + Sync, Link: Default + Send + Sync + ToString>
                + Send
                + Sync
                + 'static,
>
{
}

impl<
        T: nails::Builder<
            Handle: nails::Handle<Nail: Send + Sync, Link: Default + Send + Sync + ToString>
                        + Send
                        + Sync
                        + 'static,
        >,
    > SyncedBuilder for T
{
}

pub trait SyncedConfig<L: 'static, S: 'static>:
    'static
    + color::Config<
        'static,
        L,
        S,
        Handle: color::config::Handle<
            'static,
            L,
            S,
            Owner: slice::SliceOwner<'static, Map<'static, color::Named>: Send + Sync>,
        >,
    >
{
}

impl<T, L: 'static, S: 'static> SyncedConfig<L, S> for T where
    T: 'static
        + color::Config<
            'static,
            L,
            S,
            Handle: color::config::Handle<
                'static,
                L,
                S,
                Owner: slice::SliceOwner<'static, Map<'static, color::Named>: Send + Sync>,
            >,
        >
{
}

pub struct SyncedVerboser {
    synced: Synced<SyncData>,
    threads: usize,
    nails: usize,
}

impl SyncedVerboser {
    pub fn new(synced: Synced<SyncData>, args: &Args) -> Self {
        Self {
            synced,
            threads: match args.line_config.state {
                LineConfigState::Manual => args
                    .line_config
                    .manual
                    .iter()
                    .map(|group| group.iter().map(|item| item.cap).sum::<usize>())
                    .sum(),
                LineConfigState::Auto => args.line_config.auto.threads,
            },
            nails: match args.table_shape.shape {
                crate::args::TableShapeMode::Ellipse => args.table_shape.ellipse.get(),
                crate::args::TableShapeMode::Rectangle => args.table_shape.rectangle.get(),
            },
        }
    }

    pub fn verbose(&mut self, message: Message) {
        self.synced.lock().message = Some(message);
    }
}

impl Deref for SyncedVerboser {
    type Target = Synced<SyncData>;

    fn deref(&self) -> &Self::Target {
        &self.synced
    }
}

impl Verboser for SyncedVerboser {
    fn verbose(&mut self, message: verboser::Message) {
        self.verbose(match message {
            verboser::Message::CreatingNail(idx) => Message {
                message_type: MessageType::CreatingNail,
                message: format!(
                    "Nailing {}/{}, {}%",
                    idx,
                    self.nails,
                    (idx * 100) / self.nails
                ),
            },
            verboser::Message::Baking => Message {
                message_type: MessageType::Baking,
                message: String::from("Baking"),
            },
            verboser::Message::Dithering(idx, total) => Message {
                message_type: MessageType::Dithering,
                message: format!("Dithering {}/{}, {}%", idx, total, (idx * 100) / total),
            },
            verboser::Message::Computing(idx) => Message {
                message_type: MessageType::Computing,
                message: format!(
                    "Computing {}/{}, {}%",
                    idx,
                    self.threads,
                    (idx * 100) / self.threads
                ),
            },
        });
    }
}

#[derive(Default)]
pub struct SyncData {
    pub message: Option<Message>,
    pub computation: ComputationState,
    pub args: SyncArgs,
}

pub enum MessageType {
    LoadingImage,
    Baking,
    CreatingNail,
    Dithering,
    Computing,
    Error,
}

pub struct Message {
    message_type: MessageType,
    message: String,
}

impl Message {
    pub fn new(message_type: MessageType, message: impl ToString) -> Self {
        Self {
            message_type,
            message: message.to_string(),
        }
    }

    pub fn draw(&self, ui: &mut egui::Ui) {
        if let MessageType::Error = self.message_type {
            ui.label(
                egui::RichText::new(&self.message)
                    .italics()
                    .color(egui::Color32::RED),
            );
        } else {
            ui.label(&self.message);
        }
    }

    pub fn error(message: impl ToString) -> Self {
        Self::new(MessageType::Error, message)
    }
}

pub enum ComputationState {
    Idle,
    Running,
    Completed(Box<dyn Computation>),
}

impl Default for ComputationState {
    fn default() -> Self {
        Self::Idle
    }
}

pub trait Computation: Send + Sync {
    fn build_svg(&self, tickness: f32) -> svg::Document;

    fn build_instructions(&self) -> String;

    // fn get_line_config(&self) -> string_art::Config;
}

impl<'a, N, B, C> Computation for Cmp<N, B, C>
where
    B: Baked<Handle= N>,
    C: Send + Sync + string_art::slice::SliceOwner<'a, Item = color::Named>,
    N: nails::Handle<Link: ToString>,
{
    fn build_svg(&self, tickness: f32) -> svg::Document {
        self.build_svg(tickness)
    }

    fn build_instructions(&self) -> String {
        self.build_instructions()
    }
}
