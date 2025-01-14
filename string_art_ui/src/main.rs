#![windows_subsystem = "windows"]

use eframe::egui;
use egui::{Color32, ColorImage, Image, TextureHandle, Ui};
use image::RgbImage;
use num_traits::AsPrimitive;
use params::*;
use parking_lot::Mutex;
use rfd::FileDialog;
use string_art::{
    grid::Grid,
    nails::{self, Links},
    ColorGroupSettings, Darkness, Float,
};

mod algorithm {
    use image::RgbImage;

    use crate::Algorithm;

    pub struct Handle {
        pub algorithm: Box<dyn Algorithm>,
        pub image: RgbImage,
    }

    // impl Handle {
    //     pub fn get_rgb_image(&mut self, size: Grid<usize>) {
    //         self.image = Some(
    //             self.algorithm
    //                 .get_or_insert_with(|| {
    //                     let mut alg = self.args.create_algorithm();
    //                     alg.compute(self.args.threads);
    //                     alg
    //                 })
    //                 .build_png(size),
    //         );
    //     }
    // }
}

mod params;
use std::{
    fs::File,
    io::{BufWriter, Write},
    num::NonZero,
    sync::Arc,
};

trait Algorithm: Sync + Send {
    fn build_png(&self, resolution: Grid<usize>) -> RgbImage;

    fn build_svg(&self, tickness: f32) -> svg::Document;

    fn build_instructions(&self) -> String;

    fn compute(&mut self, threads: usize) -> Result<(), string_art::Error>;
}

impl<
        S: Float,
        N: Sync
            + Send
            + nails::Handle<Nail: Sync + Send, Scalar = S, Links: Links<Link: Sync + Send + ToString>>,
        D: Darkness<S> + Sync + Send,
    > Algorithm for string_art::Algorithm<S, N, D>
where
    usize: AsPrimitive<S>,
{
    fn build_png(&self, resolution: Grid<usize>) -> RgbImage {
        self.build_rgb(resolution)
    }

    fn build_svg(&self, tickness: f32) -> svg::Document {
        self.build_svg(tickness)
    }

    fn build_instructions(&self) -> String {
        self.build_instructions()
    }

    #[must_use]
    fn compute(&mut self, threads: usize) -> Result<(), string_art::Error> {
        self.compute(threads)
    }
}

pub struct MyApp {
    args: Args,
    state: AppState,
    algorithm: Arc<Mutex<Result<Option<algorithm::Handle>, ArgsError>>>,
    image_handle: Option<TextureHandle>,
}

enum AppState {
    Idle,
    Running,
    Completed,
    Err(String),
}
impl Default for MyApp {
    fn default() -> Self {
        MyApp {
            args: Args::new(),
            state: AppState::Idle,
            image_handle: None,
            algorithm: Arc::new(Mutex::new(Ok(None))),
        }
    }
}

impl MyApp {
    fn compute_button(&mut self, ui: &mut Ui, resolution: egui::Vec2) {
        if ui.button("Compute!").clicked() {
            *self.algorithm.lock() = Ok(None);
            self.state = AppState::Running;
            let args = self.args.clone();
            let arc = self.algorithm.clone();

            rayon::spawn(move || match args.create_algorithm() {
                Ok(mut algorithm) => match algorithm.compute(args.threads) {
                    Ok(_) => {
                        let resolution = Grid {
                            height: resolution.y as usize,
                            width: resolution.x as usize,
                        };
                        let image = algorithm.build_png(resolution);
                        *arc.lock() = Ok(Some(algorithm::Handle { algorithm, image }));
                    }
                    Err(err) => *arc.lock() = Err(ArgsError::AlgorithmError(err)),
                },
                Err(err) => *arc.lock() = Err(err),
            });
        }
    }

    fn save_svg_button(&mut self, ui: &mut Ui) {
        if ui.button("Save image").clicked() {
            if let Some(path) = FileDialog::new()
                .set_title("Save image as SVG")
                .add_filter("SVG Image", &["svg"])
                .save_file()
            {
                let alg = self.algorithm.clone();
                let tickness = self.args.tickness;
                rayon::spawn(move || {
                    let mut guard = alg.lock();
                    match &mut *guard {
                        Ok(handle) => {
                            let handle = handle.take();
                            drop(guard);
                            match handle {
                                Some(handle) => {
                                    let svg = handle.algorithm.build_svg(tickness);
                                    match File::create(path.clone()).and_then(|file| {
                                        BufWriter::new(file).write_all(svg.to_string().as_bytes())
                                    }) {
                                        Ok(()) => match open::that(path) {
                                            Ok(()) => *alg.lock() = Ok(Some(handle)),
                                            Err(error) => {
                                                *alg.lock() = Err(ArgsError::IOError(error))
                                            }
                                        },
                                        Err(error) => {
                                            *alg.lock() = Err(ArgsError::IOError(error));
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                        Err(_) => unreachable!(),
                    }
                });
            }
        }
    }

    fn save_instructions_button(&mut self, ui: &mut Ui) {
        if ui.button("Save instructions").clicked() {
            if let Some(path) = FileDialog::new()
                .set_title("Save instructions file")
                .add_filter("Plain Text Document", &["txt"])
                .save_file()
            {
                let alg = self.algorithm.clone();
                rayon::spawn(move || {
                    let mut guard = alg.lock();
                    match &mut *guard {
                        Ok(handle) => {
                            let handle = handle.take();
                            drop(guard);
                            match handle {
                                Some(handle) => {
                                    let instructions = handle.algorithm.build_instructions();
                                    match File::create(path.clone()).and_then(|file| {
                                        BufWriter::new(file).write_all(instructions.as_bytes())
                                    }) {
                                        Ok(()) => match open::that(path) {
                                            Ok(()) => *alg.lock() = Ok(Some(handle)),
                                            Err(error) => {
                                                *alg.lock() = Err(ArgsError::IOError(error))
                                            }
                                        },
                                        Err(error) => {
                                            *alg.lock() = Err(ArgsError::IOError(error));
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                        Err(_) => {}
                    }
                });
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {

                columns[0].vertical(|ui| {
                    // Threads
                    ui.horizontal(|ui| {
                        ui.label("Threads:").on_hover_text("Number of threads used when generating the image.");
                        ui.add(
                            egui::Slider::new(&mut self.args.threads, 1..=20000)
                                .clamping(egui::SliderClamping::Never),
                        );
                    });
                    // Resolution
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
                        if self.args.tickness <= 0.0 {
                            self.args.tickness = 0.1;
                        }
                    });
                });

                columns[1].vertical(|ui| {
                    //Nails
                    ui.horizontal(|ui| {
                        ui.label("Nail Count:").on_hover_text("Number of nails placed around the image.");
                        ui.add(
                            egui::Slider::new(
                                &mut self.args.nails,
                                unsafe { NonZero::new_unchecked(1) }..=unsafe {
                                    NonZero::new_unchecked(1000)
                                },
                            )
                            .clamping(egui::SliderClamping::Never),
                        )
                    });
                    // Minimum Nail Distance
                    ui.horizontal(|ui| {
                        ui.label("Minimum Nail Distance:").on_hover_text("Number of continuous nails that \
the algorithm will ignore when computing the next line.\n\nThis prevents the algorithm from drifting excessively along \
the edges of the image. Additionally, the algorithm does not consider a nail when tracing a thread if it is not the \
starting or ending nail of the line, so avoiding the tracing of edges is usually a good idea.");
                        ui.add(egui::Slider::new(
                            &mut self.args.min_nail_distance,
                            0..=(self.args.nails.get() / 2).saturating_sub(1),
                        ));
                    });
                });
            });
            ui.separator();

            //Nail shape
            ui.horizontal(|ui| {
                ui.label("Nail shape:").on_hover_text("Shape of the nail used to draw the image.");
                egui::ComboBox::from_id_salt("Nail Shape")
                    .selected_text(self.args.nail_kind)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.args.nail_kind, NailShape::Point, "Point")
                            .on_hover_text("A classic circular nail.");
                        ui.selectable_value(
                            &mut self.args.nail_kind,
                            NailShape::Circular(1.0),
                            "Circular",
                        ).on_hover_text("It is equivalent to a circular nail with a radius of zero.\n\nAlthough \
physically impossible, it significantly accelerates calculations by avoiding the need to compute tangents and the entry \
and exit points of the nail.");
                    });
                if let NailShape::Circular(radius) = &mut self.args.nail_kind {
                    ui.label("Radius:");
                    ui.add(egui::Slider::new(radius, 0.1..=10.0));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Precision:").on_hover_text("Floating-point precision.\n\nThis \
was added since the initial project. However, using double precision will increase memory costs to achieve a similar result. \
As a general rule, single precision is enough.");
                egui::ComboBox::from_label("Precision")
                    .selected_text(format!("{:?}", self.args.precision))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.args.precision, Precision::Single, "Single");
                        ui.selectable_value(&mut self.args.precision, Precision::Double, "Double");
                    });
            });

            // Darkness Mode
            ui.horizontal(|ui| {
                ui.label("Darkness Mode:").on_hover_text("An algorithm that calculates the weight decay of each pixel.\n\n\
The greater the decay of the algorithm, the more spaced out the lines of the same color will be. An algorithm with very little \
decay will generate images with a high concentration of a single color in certain regions.\n\nThe algorithm takes a floating-point \
value as input and returns another floating-point value based on the input. The returned value is expected to be greater than or \
equal to zero and less than the input value. While an algorithm that does not adhere to this premise will not produce undefined \
behavior, it will likely result in a completely nonsensical image.\n\nThe input value will be in the range [0, √140050] (the maximum \
Euclidean distance for a color in Lab format), as long as the previous condition is met.");
                egui::ComboBox::from_id_salt("Darkness Mode")
                    .selected_text(self.args.darkness_mode)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.args.darkness_mode,
                            DarknessMode::Flat(10.0),
                            "Flat",
                        ).on_hover_text("Subtracts the specified value from the input. Return zero if the result is negative.");
                        ui.selectable_value(
                            &mut self.args.darkness_mode,
                            DarknessMode::Percentage(0.93),
                            "Percentage",
                        ).on_hover_text("Multiplies the input by the specified value.");
                    });
                match &mut self.args.darkness_mode {
                    DarknessMode::Flat(flat) => {
                        ui.add(egui::Slider::new(flat, 0.1..=f32::from_bits(0x43bb1dc4)));
                    }
                    DarknessMode::Percentage(per) => {
                        ui.add(egui::Slider::new(per, 0.0..=1.0));
                    }
                }
            });

            ui.separator();

            //Palette
            ui.horizontal(|ui| {
                ui.label("Palette").on_hover_text("Colors used for the threads in the image.");
                if ui.button("+").clicked() {
                    self.args.colors.push(NamedColor {
                        name: String::from("New Color"),
                        color: [0, 0, 0],
                    });
                }
            });
            let mut removed = None;
            for (idx, color) in self.args.colors.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut color.name);
                    ui.color_edit_button_srgb(&mut color.color);
                    if ui.button("-").clicked() {
                        removed = Some(idx);
                    }
                });
            }
            if let Some(idx) = removed {
                self.args.remove_color_idx(idx);
            }

            ui.separator();
            //Groups
            ui.horizontal(|ui| {
                ui.label("Color order:");
                if self.args.colors.len() > 0 && ui.button("+").clicked() {
                    self.args.groups.push(ColorGroupSettings::new((0..self.args.colors.len()).into_iter().collect(), 0.5));
                }
            });
            let mut removed_group = None;
            for (group_idx, group) in self.args.groups.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", group_idx + 1));
                    if ui.button("+").clicked() {
                        group.push(0);
                    }
                    if ui.button("-").clicked() {
                        removed_group = Some(group_idx);
                    }
                    ui.add(egui::Slider::new(&mut group.weight, 0.0..=1.0).show_value(false));
                    let mut removed_element = None;
                    for (element_idx, color_idx) in group.colors.iter_mut().enumerate() {
                        egui::ComboBox::from_id_salt(color_idx as *mut _)
                            .selected_text(unsafe {
                                &self.args.colors.get_unchecked(*color_idx).name
                            })
                            .show_ui(ui, |ui| {
                                for (idx, color) in self.args.colors.iter().enumerate() {
                                    ui.selectable_value(color_idx, idx, &color.name);
                                }
                            });
                            if ui.button("-").clicked() {
                                removed_element = Some(element_idx);
                            }
                    }
                    if let Some(idx) = removed_element{
                        group.colors.remove(idx);
                    }
                });
            }
            if let Some(idx) = removed_group{
                self.args.groups.remove(idx);
            }
            ui.separator();
            //Image picker
            ui.horizontal(|ui| {
                if ui.button("Select Image").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter(
                            "All compatible images",
                            &[
                                "bmp", "dds", "ff", "gif", "hdr", "ico", "jpg", "jpeg", "exr",
                                "png", "pbm", "pgm", "ppm", "pam", "qoi", "tga", "tiff", "tif",
                                "webp",
                            ],
                        ) // Filtro que incluye todas las extensiones compatibles
                        .add_filter("Bitmap Image", &["bmp"])
                        .add_filter("DirectDraw Surface", &["dds"])
                        .add_filter("Farbfeld Image", &["ff"])
                        .add_filter("Graphics Interchange Format", &["gif"])
                        .add_filter("High Dynamic Range Image", &["hdr"])
                        .add_filter("Icon File", &["ico"])
                        .add_filter("JPEG Image", &["jpg", "jpeg"])
                        .add_filter("OpenEXR Image", &["exr"])
                        .add_filter("Portable Network Graphics", &["png"])
                        .add_filter("Portable Any Map", &["pbm", "pgm", "ppm", "pam"])
                        .add_filter("QOI Image", &["qoi"])
                        .add_filter("Targa Image", &["tga"])
                        .add_filter("Tagged Image File Format", &["tiff", "tif"])
                        .add_filter("WebP Image", &["webp"])
                        .pick_file()
                    {
                        self.args.file_path = Some(path.display().to_string());
                    }
                }
                if let Some(file_path) = &self.args.file_path {
                    ui.label(file_path);
                    if ui.button("-").clicked() {
                        self.args.file_path = None;
                    }
                }
            });

            ui.separator();
            //Generator
            match &self.state {
                AppState::Idle => {
                    self.compute_button(ui, ui.available_size());
                }
                AppState::Running => {
                    let mut guard = self.algorithm.lock();
                    match &mut *guard {
                        Ok(handle) => match handle {
                            Some(handle) => {
                                self.state = AppState::Completed;
                                self.image_handle = Some(ctx.load_texture(
                                    "Output",
                                    ColorImage::from_rgb(
                                        [
                                            handle.image.width() as usize,
                                            handle.image.height() as usize,
                                        ],
                                        handle.image.as_raw(),
                                    ),
                                    Default::default(),
                                ));
                            }
                            None => {
                                drop(guard);
                                ui.spinner();
                            }
                        },
                        Err(err) => self.state = AppState::Err(err.to_string()),
                    }
                }
                AppState::Completed => {
                    let resolution = ui.available_size();
                    ui.horizontal(|ui| {
                        self.compute_button(ui, resolution);
                        self.save_svg_button(ui);
                        self.save_instructions_button(ui);
                    });
                    let texture = self.image_handle.as_ref().unwrap();
                    ui.add(Image::new(texture));
                }
                AppState::Err(err) => {
                    let err = err.to_string();
                    let resolution = ui.available_size();
                    ui.horizontal(|ui| {
                        self.compute_button(ui, resolution);
                        ui.label(egui::RichText::new(err).color(Color32::from_rgb(255, 0, 0)));
                    });
                }
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "String Art",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
    .unwrap();
}
