use crate::synced::{Message, MessageType, SyncedBuilder, SyncedConfig, SyncedVerboser};
use darkness_mode::DarknessType;
use num_traits::AsPrimitive;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::{
    num::NonZero,
    ops::{Deref, DerefMut},
};
use string_art::{
    color::{
        self,
        config::multi as config,
        Rgb,
    },
    darkness::{Darkness, FlatDarkness, PercentageDarkness},
    nails::{self, Circular},
    BakedNailTable, Float, Image, NailTable,
};

use super::synced::Computation;

mod darkness_mode;
mod line_config;
mod nail_shape;
mod precision;
mod table_shape;

pub use darkness_mode::DarknessMode;
pub use line_config::{LineConfig, LineConfigState};
pub use nail_shape::NailShape;
pub use precision::Precision;
pub use table_shape::{TableShape, TableShapeMode};

#[derive(Clone, Serialize, Deserialize)]
pub struct Args {
    inner: ArgsP,
    pub precision: Precision,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            precision: Precision::Single,
        }
    }
}

impl Deref for Args {
    type Target = ArgsP;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Args {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArgsP {
    inner: ArgsD,
    /// Darkness mode of processing.
    pub darkness_mode: DarknessMode,
}

impl Default for ArgsP {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            darkness_mode: DarknessMode {
                flat: 0.3,
                percentage: 0.9,
                mode: DarknessType::Flat,
            },
        }
    }
}

impl Deref for ArgsP {
    type Target = ArgsD;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ArgsP {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArgsD {
    inner: ArgsN,
    pub nail_shape: NailShape,
}

impl Default for ArgsD {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            nail_shape: NailShape::Circular(1.0),
        }
    }
}

impl Deref for ArgsD {
    type Target = ArgsN;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ArgsD {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArgsN {
    inner: ArgsF,
    /// Colors of the palete. Acepta sintaxis del tipo "white:FFF", "white:FFFFFF",
    /// "black:0,0,0" y varios colores comunes identificados directamente por su nombre
    pub palette: Vec<color::Named>,
    pub line_config: LineConfig,
}

impl Default for ArgsN {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            palette: vec![color::Named::new(String::from("Black"), Rgb(0, 0, 0))],
            line_config: LineConfig::new(
                config::Manual::new(vec![config::manual::Group::new(vec![
                    config::manual::Item::new(0, 12000),
                ])]),
                config::Auto::new(vec![config::auto::Group::new(vec![0], 0.5)], 12000),
                4000,
                LineConfigState::Auto,
            ),
        }
    }
}

impl Deref for ArgsN {
    type Target = ArgsF;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ArgsN {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArgsF {
    /// Input file path.
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub file_path: Option<String>,
    /// Number of nails surrounding the image.    
    //pub nails: NonZero<usize>,
    pub table_shape: TableShape,
    /// Size in pixels of the longest side of the image.
    pub resolution: NonZero<u32>,
    /// Darkness mode of processing.
    pub contrast: f32,
    pub blur_radius: usize,
    /// Minimum nail count between linked nails.
    pub min_nail_distance: usize,
    pub tickness: f32,
}

impl Default for ArgsF {
    fn default() -> Self {
        Self {
            file_path: None,
            table_shape: TableShape {
                rectangle: unsafe { NonZero::new_unchecked(512) },
                ellipse: unsafe { NonZero::new_unchecked(512) },
                shape: TableShapeMode::Ellipse,
            },
            resolution: unsafe { NonZero::new_unchecked(1000) },
            contrast: 0.5,
            blur_radius: 4,
            min_nail_distance: 20,
            tickness: 0.25,
        }
    }
}

impl Args {
    pub fn palette_form(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Palette")
                .on_hover_text("Colors used for the threads in the image.");
            if ui.button("+").clicked() {
                self.palette
                    .push(color::Named::new(String::from("New Color"), Rgb(0, 0, 0)));
            }
        });
        let mut removed = None;
        let palette_len = self.palette.len();
        for (idx, color) in self.palette.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut color.name);
                let mut arr_col = color.value.into();
                ui.color_edit_button_srgb(&mut arr_col);
                color.value = arr_col.into();
                if palette_len > 1 && ui.button("🗑").clicked() {
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

    pub fn compute(self, verboser: &mut SyncedVerboser) -> Result<Box<dyn Computation>, Error> {
        match self.precision {
            Precision::Single => self.inner.compute::<f32>(verboser),
            Precision::Double => self.inner.compute::<f64>(verboser),
        }
    }
}
impl ArgsP {
    fn compute<S: Float>(self, verboser: &mut SyncedVerboser) -> Result<Box<dyn Computation>, Error>
    where
        f32: AsPrimitive<S>,
        usize: AsPrimitive<S>,
        u8: AsPrimitive<S>,
    {
        match self.darkness_mode.mode {
            DarknessType::Flat => self
                .inner
                .compute::<S, _>(FlatDarkness(self.darkness_mode.flat.as_()), verboser),
            DarknessType::Percentage => self.inner.compute::<S, _>(
                PercentageDarkness(self.darkness_mode.percentage.as_()),
                verboser,
            ),
        }
    }
}

impl ArgsD {
    fn compute<S, D>(
        self,
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
                self.inner
                    .compute(darkness, Circular::new(radius.as_()), verboser)
            }
            NailShape::Point => Err(Error::UnimplementedFeature("Point nail")),
        }
    }
}

impl ArgsN {
    fn compute<D, N>(
        self,
        darkness: D,
        handle: N,
        verboser: &mut SyncedVerboser,
    ) -> Result<Box<dyn Computation>, Error>
    where
        usize: AsPrimitive<N::Scalar>,
        u8: AsPrimitive<N::Scalar>,
        f32: AsPrimitive<N::Scalar>,
        D: Darkness<N::Scalar> + Send + Sync + 'static,
        N: SyncedBuilder,
    {
        let palette: Vec<_> = self
            .palette
            .iter()
            .map(|color| {
                string_art::color::mapping::State::new(color.clone(), 0, Default::default())
            })
            .collect();

        match &self.line_config.state {
            LineConfigState::Manual => self.inner.compute(
                darkness,
                handle,
                verboser,
                config::Config::new(palette, self.line_config.manual),
            ),
            LineConfigState::Auto => self.inner.compute(
                darkness,
                handle,
                verboser,
                config::Config::new(palette, self.line_config.auto),
            ),
        }
    }

    pub fn line_form(&mut self, ui: &mut egui::Ui) {
        self.line_config.form(ui, &self.palette);
    }
}

impl ArgsF {
    fn compute<D, N, C>(
        self,
        darkness: D,
        handle: N,
        verboser: &mut SyncedVerboser,
        config: C,
    ) -> Result<Box<dyn Computation>, Error>
    where
        usize: AsPrimitive<N::Scalar>,
        u8: AsPrimitive<N::Scalar>,
        f32: AsPrimitive<N::Scalar>,
        D: Darkness<N::Scalar> + Send + Sync + 'static,
        N: SyncedBuilder,
        C: SyncedConfig<<N::Handle as nails::Handle>::Link, N::Scalar>,
    {
        match &self.file_path {
            Some(file_path) => {
                verboser.verbose(Message::new(MessageType::LoadingImage, "Loading image..."));

                let image: Image<N::Scalar> = image::open(file_path)
                    .map_err(|err| Error::Image(err))?
                    .resize(
                        self.resolution.get(),
                        self.resolution.get(),
                        image::imageops::FilterType::Lanczos3,
                    )
                    .into();
                let table = match self.table_shape.shape {
                    TableShapeMode::Ellipse => NailTable::ellipse(
                        *image.grid(),
                        handle,
                        self.table_shape.ellipse.get(),
                        verboser,
                    ),
                    TableShapeMode::Rectangle => NailTable::square(
                        *image.grid(),
                        handle,
                        self.table_shape.rectangle.get(),
                        verboser,
                    )
                    .map_err(|err| Error::Computation(Box::new(err)))?,
                };

                //let a= string_art::color::Config::into_color_handle(config, &image, 100, self.blur_radius, self.contrast.as_()).unwrap();
                let a = string_art::compute(
                    BakedNailTable::new(table, self.min_nail_distance).map_err(|err| Error::Computation(Box::new(err)))?,
                    &image,
                    config,
                    darkness,
                    self.contrast.as_(),
                    self.blur_radius,
                    verboser,
                );

                match a {
                    Ok(computation) => {
                        let cmp = computation;
                        Ok(Box::new(cmp))
                    }
                    Err(err) => Err(Error::Computation(Box::new(err))),
                }
            }
            None => Err(Error::MissingFilePath),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing file path.")]
    MissingFilePath,

    #[error(transparent)]
    Computation(Box<dyn std::error::Error>),

    #[error("Unimplemented feature: {0}.")]
    UnimplementedFeature(&'static str),

    #[error(transparent)]
    Image(image::ImageError),
}
