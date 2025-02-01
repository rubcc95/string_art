#![windows_subsystem = "windows"]

use args::Args;
use egui::{IconData, ViewportBuilder};
use rfd::FileDialog;
use std::{
    fs::File, io::{BufWriter, Read, Write}, mem, num::NonZero, path::Path, sync::Arc
};
use synced::{ComputationState, Message, SyncData, Synced, SyncedVerboser};

fn config_path() -> Option<std::path::PathBuf> {    
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("LOCALAPPDATA").as_ref().map(|path| Path::new(path).join("string_art").join("config.sac"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::var_os("HOME").as_ref().map(|path| Path::new(path).join(".config").join("string_art").join("config.sac"))
        }            
}

enum ArgsState {
    Ready,
    Waiting,
}

impl Default for ArgsState {
    fn default() -> Self {
        Self::Ready
    }
}

enum SyncArgs {
    Waiting,
    Done(Option<Args>),
}

impl Default for SyncArgs {
    fn default() -> Self {
        Self::Done(None)
    }
}

#[derive(Default)]
struct App {
    args: Args,
    args_state: ArgsState,
    sync_data: Synced<SyncData>,
    message: Option<Message>,
    computation: ComputationState,
}

impl App {
    fn new(args: Args) -> Self {
        Self {
            args,
            ..Default::default()
        }
    }

    fn with_error(err: impl ToString) -> Self{
        Self{
            message: Some(Message::error(err)),
            ..Default::default()
        }
    }

    fn compute_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("Compute!").clicked() {
            self.sync_data.lock().computation = ComputationState::Running;
            self.computation = ComputationState::Running;
            let args = self.args.clone();
            let mut verboser = SyncedVerboser::new(self.sync_data.clone(), &args);
            rayon::spawn(move || match args.create_algorithm(&mut verboser) {
                Ok(algorithm) => {
                    verboser.lock().computation = ComputationState::Completed(algorithm)
                }
                Err(err) => {
                    let mut synced = verboser.lock();
                    synced.message = Some(Message::error(err));
                    synced.computation = ComputationState::Idle;
                }
            });
        }
    }

    fn main_menu(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {

            columns[0].vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Resolution:").on_hover_text("Resolution of the largest side of the image \
when being computed.\n\nA higher resolution implies greater detail. However, excessively increasing this value does not \
lead to significant improvements and exponentially increases computational costs. As a general rule, a value of 1000 \
is more than sufficient for square images.\n\nModifying this value can significantly alter the result, as higher \
resolution values will require a darkness algorithm with a steeper gradient to maintain a coherent outcome.");
                    ui.add(
                        egui::Slider::new(
                            &mut self.args.resolution,
                            unsafe { NonZero::new_unchecked(1) }..=unsafe {
                                NonZero::new_unchecked(2000)
                            },
                        )
                        .clamping(egui::SliderClamping::Never),
                    );
                });
                // Tickness
                ui.horizontal(|ui| {
                    ui.label("Thread Tickness:");
                    ui.add(
                        egui::Slider::new(&mut self.args.tickness, 0.1..=2.0)
                            .clamping(egui::SliderClamping::Never),
                    );
                    if self.args.tickness < 0.001 {
                        self.args.tickness = 0.001;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Contrast:");
                    ui.add(
                        egui::Slider::new(&mut self.args.contrast, 0.0..=1.0)
                    );
                });
            });

            columns[1].vertical(|ui| {
                //Nails
                self.args.table_shape.form(ui);
                // Minimum Nail Distance
                ui.horizontal(|ui| {
                    ui.label("Minimum Nail Distance:").on_hover_text("Number of continuous nails that \
the algorithm will ignore when computing the next line.\n\nThis prevents the algorithm from drifting excessively along \
the edges of the image. Additionally, the algorithm does not consider a nail when tracing a thread if it is not the \
starting or ending nail of the line, so avoiding the tracing of edges is usually a good idea.");
                    ui.add(egui::Slider::new(
                        &mut self.args.min_nail_distance,
                        0..=(match self.args.table_shape.shape{
                            args::TableShapeMode::Ellipse => self.args.table_shape.ellipse,
                            args::TableShapeMode::Rectangle => self.args.table_shape.rectangle,
                        }.get() / 2).saturating_sub(1),
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Blur Radius:");
                    ui.add(
                        egui::Slider::new(&mut self.args.blur_radius, 0..=10)
                            .clamping(egui::SliderClamping::Never),
                    );
                    if self.args.tickness <= 0.0 {
                        self.args.tickness = 0.1;
                    }
                });
            });
        });

        ui.separator();

        self.args.nail_shape.form(ui);
        self.args.precision.form(ui);
        self.args.darkness_mode.form(ui);

        ui.separator();

        self.args.palette_form(ui);

        ui.separator();

        self.args.line_config.form(ui, &self.args.palette);

        ui.separator();

        self.args.image_picker(ui);
        ui.separator();

        ui.horizontal(|ui| {
            match &self.computation {
                ComputationState::Idle => {
                    self.compute_button(ui);
                }
                ComputationState::Running => {
                    ui.spinner();
                    let mut synced = self.sync_data.lock();
                    let message = synced.message.take();
                    if let Some(message) = message {
                        self.message = Some(message);
                    }
                    match &mut synced.computation {
                        ComputationState::Running => {}
                        ComputationState::Idle => {
                            self.computation =
                                mem::replace(&mut synced.computation, ComputationState::Idle);
                        }
                        ComputationState::Completed(computation) => {
                            self.args.line_config.manual = computation.get_line_config();
                            self.computation =
                                mem::replace(&mut synced.computation, ComputationState::Idle);
                        }
                    }
                }
                ComputationState::Completed(_) => {
                    self.compute_button(ui);
                    if ui.button("Save image").clicked() {
                        self.sync_data.lock().computation = ComputationState::Running;
                        let synced = self.sync_data.clone();
                        let computation =
                            match mem::replace(&mut self.computation, ComputationState::Running) {
                                ComputationState::Completed(computation) => computation,
                                _ => unsafe { core::hint::unreachable_unchecked() },
                            };
                        let tickness = self.args.tickness;
                        rayon::spawn(move || {
                            match FileDialog::new()
                                .set_title("Save image as SVG")
                                .add_filter("SVG Image", &["svg"])
                                .save_file()
                            {
                                Some(path) => {
                                    let svg = computation.build_svg(tickness);
                                    match File::create(path.clone()).and_then(|file| {
                                        BufWriter::new(file).write_all(svg.to_string().as_bytes())
                                    }) {
                                        Ok(_) => match open::that(path) {
                                            Ok(_) => synced.lock(),
                                            Err(err) => {
                                                let mut lock = synced.lock();
                                                lock.message = Some(Message::error(err));
                                                lock
                                            }
                                        },
                                        Err(err) => {
                                            let mut lock = synced.lock();
                                            lock.message = Some(Message::error(err));
                                            lock
                                        }
                                    }
                                    .computation = ComputationState::Completed(computation);
                                }
                                None => {
                                    synced.lock().computation =
                                        ComputationState::Completed(computation)
                                }
                            }
                        });
                    }
                    //ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save instructions").clicked() {
                        self.sync_data.lock().computation = ComputationState::Running;
                        let synced = self.sync_data.clone();
                        let computation =
                            match mem::replace(&mut self.computation, ComputationState::Running) {
                                ComputationState::Completed(computation) => computation,
                                _ => unsafe { core::hint::unreachable_unchecked() },
                            };
                        rayon::spawn(move || {
                            match FileDialog::new()
                                .set_title("Save instructions file")
                                .add_filter("Plain Text Document", &["txt"])
                                .save_file()
                            {
                                Some(path) => {
                                    let instructions = computation.build_instructions();
                                    match File::create(path.clone()).and_then(|file| {
                                        BufWriter::new(file).write_all(instructions.as_bytes())
                                    }) {
                                        Ok(_) => match open::that(path) {
                                            Ok(_) => synced.lock(),
                                            Err(err) => {
                                                let mut lock = synced.lock();
                                                lock.message = Some(Message::error(err));
                                                lock
                                            }
                                        },
                                        Err(err) => {
                                            let mut lock = synced.lock();
                                            lock.message = Some(Message::error(err));
                                            lock
                                        }
                                    }
                                    .computation = ComputationState::Completed(computation);
                                }
                                None => {
                                    synced.lock().computation =
                                        ComputationState::Completed(computation)
                                }
                            }
                        });
                    }
                    //});
                }
            }
            if let Some(message) = &self.message {
                message.draw(ui);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Load Config").clicked() {
                    self.args_state = ArgsState::Waiting;
                    self.sync_data.lock().args = SyncArgs::Waiting;
                    let data = self.sync_data.clone();
                    rayon::spawn(move || {
                        let mut buffer = Vec::new();
                        if let Some(path) = FileDialog::new()
                            .set_title("Load configuration file")
                            .add_filter("String Art Configuration File", &["sac"])
                            .pick_file()
                        {
                            let (mut data, args) = match File::open(path)
                                .and_then(|mut file| file.read_to_end(&mut buffer))
                            {
                                Ok(_) => match bincode::deserialize::<Args>(&buffer) {
                                    Ok(args) => (data.lock(), Some(args)),
                                    Err(_) => {
                                        let mut data = data.lock();
                                        data.message = Some(Message::error(
                                            "Failed to read the file: the file is corrupted.",
                                        ));
                                        (data, None)
                                    }
                                },
                                Err(err) => {
                                    let mut data = data.lock();
                                    data.message = Some(Message::error(err));
                                    (data, None)
                                }
                            };
                            data.args = SyncArgs::Done(args)
                        }
                    })
                }
                if ui.button("Save Config").clicked() {
                    let args = self.args.clone();
                    let data = self.sync_data.clone();
                    rayon::spawn(move || {
                        let json = bincode::serialize(&args).unwrap();
                        let path = FileDialog::new()
                            .set_title("Save configuration file")
                            .add_filter("String Art Configuration File", &["sac"])
                            .save_file();
                        if let Some(path) = path {
                            if let Err(err) = File::create(path)
                                .and_then(|file| BufWriter::new(file).write_all(&json))
                            {
                                data.lock().message = Some(Message::error(err));
                            }
                        }
                    });
                }
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let ArgsState::Waiting = self.args_state {
                let mut data = self.sync_data.lock();
                if let SyncArgs::Done(new_args) = &mut data.args {
                    if let Some(args) = new_args.take() {
                        self.args = args;
                    }
                    self.message = data.message.take();
                    self.args_state = ArgsState::Ready;
                }
            }
            egui::ScrollArea::vertical().show(ui, |ui|{ 
                self.main_menu(ui);
            })
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let json = bincode::serialize(&self.args).unwrap();
        if let Some(path) = config_path() {
            if path.parent().is_some_and(|parent| parent.exists() || { std::fs::create_dir_all(&parent).is_ok() }){           
                match File::create(path.clone()){
                    Ok(mut file) =>  {                   
                        file.write_all(&json).unwrap();
                    },
                    Err(err) => {
                        let string = err.to_string();
                        eprintln!("Failed to save configuration file: {}", string);
                    }
                }
            }            
        }
    }
}

fn main() {
    eframe::run_native(
        "String Art",
        eframe::NativeOptions {
            viewport: ViewportBuilder {
                icon: Some(Arc::new(IconData {
                    rgba: include_bytes!("../assets/icon.rgba")
                        .iter()
                        .copied()
                        .collect(),
                    width: 384,
                    height: 384,
                })),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|_cc| {
            let path = std::env::args()
                .nth(1)
                .map(std::path::PathBuf::from)
                .or_else(config_path);

            Ok(Box::new(match path{
                Some(path) => {
                    let mut buffer = Vec::new();                    
                    match File::open(path){
                        Ok(mut file) => match file.read_to_end(&mut buffer){
                            Ok(_) => {
                                match bincode::deserialize(&buffer){
                                    Ok(args) => App::new(args),
                                    Err(_) => App::with_error("Configuration file is corrupted, reverting to default settings."),
                                }
                            },
                            Err(err) => App::with_error(err),
                        },
                        Err(_) => App::new(Args::default()),
                    }
                },
                None => App::new(Args::default()),
            }))
        }),
    )
    .unwrap();
}

mod args;
mod synced;
