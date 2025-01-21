use crate::synced::{Message, MessageType, SyncedVerboser};
use num_traits::AsPrimitive;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use string_art::{
    auto_line_config::{AutoLineConfig, AutoLineGroupConfig},
    darkness::{Darkness, FlatDarkness, PercentageDarkness},
    line_config::{LineGroupConfig, LineItemConfig},
    nails::{self, Circular},
    AsLab, ColorConfig, Float, Image, Lab, NailTable,
};

use super::synced::Computation;

mod arg_line_count;
mod darkness_mode;
mod nail_shape;
mod precision;

pub use arg_line_count::{ArgLineCount, ArgLineCountState};
pub use darkness_mode::DarknessMode;
pub use nail_shape::NailShape;
pub use precision::Precision;

#[derive(Clone, Serialize, Deserialize)]
pub struct Args {
    /// Input file path.
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub file_path: Option<String>,
    /// Number of nails surrounding the image.    
    pub nails: NonZero<usize>,
    pub nail_shape: NailShape,
    /// Size in pixels of the longest side of the image.
    pub resolution: NonZero<u32>,
    /// Precision of calculations (Single/Double).
    pub precision: Precision,
    /// Darkness mode of processing.
    pub darkness_mode: DarknessMode,
    /// Minimum nail count between linked nails.
    pub min_nail_distance: usize,
    /// Colors of the palete. Acepta sintaxis del tipo "white:FFF", "white:FFFFFF",
    /// "black:0,0,0" y varios colores comunes identificados directamente por su nombre
    pub palette: Vec<NamedColor>,
    pub tickness: f32,
    pub line_config: ArgLineCount,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            file_path: None,
            nails: unsafe { NonZero::new_unchecked(512) },
            nail_shape: NailShape::Circular(1.0),
            resolution: unsafe { NonZero::new_unchecked(1000) },
            precision: Precision::Single,
            darkness_mode: DarknessMode::Percentage(0.93),
            min_nail_distance: 20,
            palette: vec![NamedColor {
                name: String::from("Black"),
                color: (0, 0, 0),
            }],
            line_config: ArgLineCount::new(
                string_art::LineConfig::new(vec![LineGroupConfig::new(vec![LineItemConfig::new(
                    0, 4000,
                )])]),
                AutoLineConfig::new(vec![AutoLineGroupConfig::new(vec![0], 0.5)], 4000),
                ArgLineCountState::Auto,
            ),
            tickness: 1.0,
        }
    }
}

impl Args {
    pub fn palette_form(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Palette")
                .on_hover_text("Colors used for the threads in the image.");
            if ui.button("+").clicked() {
                self.palette.push(NamedColor {
                    name: String::from("New Color"),
                    color: (0, 0, 0),
                });
            }
        });
        let mut removed = None;
        for (idx, color) in self.palette.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut color.name);
                let mut arr_col = color.color.into();
                ui.color_edit_button_srgb(&mut arr_col);
                color.color = arr_col.into();
                if ui.button("-").clicked() {
                    removed = Some(idx);
                }
            });
        }
        if let Some(idx) = removed {
            self.remove_color_idx(idx);
        }
    }

    pub fn image_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Select Image").clicked() {
                if let Some(path) = FileDialog::new()
                    .add_filter(
                        "All compatible images",
                        &[
                            "bmp", "dds", "ff", "gif", "hdr", "ico", "jpg", "jpeg", "exr", "png",
                            "pbm", "pgm", "ppm", "pam", "qoi", "tga", "tiff", "tif", "webp",
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
                    self.file_path = Some(path.display().to_string());
                }
            }
            if let Some(file_path) = &self.file_path {
                ui.label(file_path);
                if ui.button("-").clicked() {
                    self.file_path = None;
                }
            }
        });
    }

    pub fn remove_color_idx(&mut self, index: usize) {
        self.line_config.manual.retain_mut(|group| {
            group.retain_mut(|item| match index.cmp(&item.color_idx) {
                std::cmp::Ordering::Less => {
                    item.color_idx -= 1;
                    true
                }
                std::cmp::Ordering::Equal => false,
                std::cmp::Ordering::Greater => true,
            });
            group.len() > 0
        });
        self.line_config.auto.retain_mut(|group| {
            group.retain_mut(|item| match index.cmp(item) {
                std::cmp::Ordering::Less => {
                    *item -= 1;
                    true
                }
                std::cmp::Ordering::Equal => false,
                std::cmp::Ordering::Greater => true,
            });
            group.len() > 0
        });
        self.palette.remove(index);
    }

    pub fn create_algorithm(
        &self,
        verboser: &mut SyncedVerboser,
    ) -> Result<Box<dyn Computation>, Error> {
        match self.precision {
            Precision::Single => self.create_algorithm_with_scalar::<f32>(verboser),
            Precision::Double => self.create_algorithm_with_scalar::<f64>(verboser),
        }
    }

    fn create_algorithm_with_scalar<S: Float>(
        &self,
        verboser: &mut SyncedVerboser,
    ) -> Result<Box<dyn Computation>, Error>
    where
        f32: AsPrimitive<S>,
        usize: AsPrimitive<S>,
        u8: AsPrimitive<S>,
    {
        match self.darkness_mode {
            DarknessMode::Flat(flat) => {
                self.create_algorithm_with_darkness::<S, _>(FlatDarkness(flat.as_()), verboser)
            }
            DarknessMode::Percentage(per) => {
                self.create_algorithm_with_darkness::<S, _>(PercentageDarkness(per.as_()), verboser)
            }
        }
    }

    fn create_algorithm_with_darkness<S, D>(
        &self,
        darkness: D,
        verboser: &mut SyncedVerboser,
    ) -> Result<Box<dyn Computation>, Error>
    where
        usize: AsPrimitive<S>,
        f32: AsPrimitive<S>,
        u8: AsPrimitive<S>,
        S: Float,
        D: Darkness<S> + Send + Sync + 'static,
    {
        match self.nail_shape {
            NailShape::Circular(radius) => {
                self.create_algorithm_with_nails(darkness, Circular::new(radius.as_()), verboser)
            }
            NailShape::Point => Err(Error::UnimplementedFeature("Point nail kind")),
        }
    }

    fn create_algorithm_with_nails<D, N>(
        &self,
        darkness: D,
        handle: N,
        verboser: &mut SyncedVerboser,
    ) -> Result<Box<dyn Computation>, Error>
    where
        usize: AsPrimitive<N::Scalar>,
        u8: AsPrimitive<N::Scalar>,
        f32: AsPrimitive<N::Scalar>,
        D: Darkness<N::Scalar> + Send + Sync + 'static,
        N: nails::Builder<
            Scalar: Float,
            Handle: nails::Handle<Nail: Send + Sync, Link: Default + Send + Sync + ToString>
                        + Send
                        + Sync
                        + 'static,
        >,
    {
        match &self.file_path {
            Some(file_path) => {
                verboser.verbose(Message::new(MessageType::LoadingImage, "Loading image..."));
                let image: Image<N::Scalar> = image::open(file_path)
                    .map_err(|err| Error::ImageError(err))?
                    .resize(
                        self.resolution.get(),
                        self.resolution.get(),
                        image::imageops::FilterType::Lanczos3,
                    )
                    .into();
                let table = NailTable::ellipse(*image.grid(), handle, self.nails.get(), verboser);
                match string_art::Algorithm::new(
                    table,
                    self.min_nail_distance,
                    &image,
                    self.palette.iter().map(|color| {
                        ColorConfig::new(
                            color.name.clone(),
                            color.color.into(),
                            0,
                            Default::default(),
                        )
                    }),
                    darkness,
                    &self.line_config,
                    verboser,
                ) {
                    Ok(algorithm) => Ok(Box::new(algorithm)),
                    Err(err) => Err(Error::AlgorithmError(Box::new(err))),
                }
            }
            None => Err(Error::MissingFilePath),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedColor {
    pub name: String,
    pub color: (u8, u8, u8),
}

impl<S: Float> AsLab<S> for NamedColor
where
    u8: AsPrimitive<S>,
{
    fn as_lab(&self) -> Lab<S> {
        self.color.as_lab()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing file path.")]
    MissingFilePath,

    #[error(transparent)]
    AlgorithmError(Box<dyn std::error::Error>),

    #[error("Unimplemented feature: {0}.")]
    UnimplementedFeature(&'static str),

    #[error(transparent)]
    ImageError(image::ImageError),
}
