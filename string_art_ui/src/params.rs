use std::num::NonZero;

use egui::{RichText, WidgetText};
use num_traits::AsPrimitive;
use string_art::{
    nails::{self, Circular}, ColorGroupSettings, ColorMapSettings, Darkness, FlatDarkness, Float, PercentageDarkness, Table
};
use thiserror::Error;
use crate::Algorithm;

#[derive(Clone)]
pub struct Args {
    /// Input file path.
    pub file_path: Option<String>,
    /// Number of nails surrounding the image.    
    pub nails: NonZero<usize>,
    pub nail_kind: NailShape,
    /// Size in pixels of the longest side of the image.
    pub resolution: NonZero<u32>,
    /// Precision of calculations (Single/Double).
    pub precision: Precision,
    /// Darkness mode of processing.
    pub darkness_mode: DarknessMode,
    /// Minimum nail count between linked nails.
    pub min_nail_distance: usize,
    pub threads: usize,
    /// Colors of the palete. Acepta sintaxis del tipo "white:FFF", "white:FFFFFF",
    /// "black:0,0,0" y varios colores comunes identificados directamente por su nombre
    pub colors: Vec<NamedColor>,
    pub tickness: f32,
    pub groups: Vec<ColorGroupSettings<Vec<usize>, f32>>,
}

impl Args {
    pub fn create_algorithm(&self) -> Result<Box<dyn Algorithm>, ArgsError> {
        match self.precision {
            Precision::Single => self.create_algorithm_with_scalar::<f32>(),
            Precision::Double => self.create_algorithm_with_scalar::<f64>(),
        }
    }

    pub fn remove_color_idx(&mut self, index: usize) {
        self.groups.retain_mut(|sub_group|{
            sub_group.retain_mut(|element|{
                match index.cmp(element){
                    std::cmp::Ordering::Less => {
                        *element -= 1;
                        true
                    },
                    std::cmp::Ordering::Equal => false,
                    std::cmp::Ordering::Greater => true,
                }
            });
            sub_group.len() > 0
        });
        self.colors.remove(index);
    }

    fn create_algorithm_with_scalar<S: Float + Send + Sync>(
        &self,
    ) -> Result<Box<dyn Algorithm>, ArgsError>
    where
        f32: AsPrimitive<S>,
        usize: AsPrimitive<S>,
        u8: AsPrimitive<S>,
    {
        match self.darkness_mode {
            DarknessMode::Flat(flat) => {
                self.create_algorithm_with_darkness::<S, _>(FlatDarkness(flat.as_()))
            }
            DarknessMode::Percentage(per) => {
                self.create_algorithm_with_darkness::<S, _>(PercentageDarkness(per.as_()))
            }
        }
    }

    fn create_algorithm_with_darkness<S, D>(
        &self,
        darkness: D,
    ) -> Result<Box<dyn Algorithm>, ArgsError>
    where
        usize: AsPrimitive<S>,
        f32: AsPrimitive<S>,
        u8: AsPrimitive<S>,
        S: Float,
        D: Darkness<S> + Send + Sync + 'static,
    {
        match self.nail_kind {
            NailShape::Circular(radius) => {
                self.create_algorithm_with_nails::<S, _, _>(darkness, Circular::new(radius.as_()))
            }
            NailShape::Point => Err(ArgsError::UnimplementedFeature("Point nail kind")),
        }
    }

    fn create_algorithm_with_nails<S, D, N>(
        &self,
        darkness: D,
        handle: N,
    ) -> Result<Box<dyn Algorithm>, ArgsError>
    where
        usize: AsPrimitive<S>,
        u8: AsPrimitive<S>,
        f32: AsPrimitive<S>,
        S: Float + Send + Sync,
        D: Darkness<S> + Send + Sync + 'static,
        N: nails::Builder<
            Scalar = S,
            Handle: nails::Handle<Nail: Send + Sync, Link: Default + Send + Sync + ToString>
                        + Send
                        + Sync
                        + 'static,
        >,
    {
        match &self.file_path {
            Some(file_path) => {
                let image = image::open(file_path).map_err(|err| ArgsError::ImageError(err))?;
                let table = Table::ellipse(image, handle, self.nails.get());
                match string_art::Algorithm::new(                    
                    table,
                    self.colors.iter().map(|color| {
                        ColorMapSettings::new(
                            color.name.clone(),
                            color.color.into(),
                            0,
                            Default::default(),
                        )
                    }),
                    self.min_nail_distance,
                    darkness,
                    &self.groups,
                    self.threads,
                ) {
                    Ok(algorithm) => Ok(Box::new(algorithm)),
                    Err(err) => Err(ArgsError::AlgorithmError(err)),
                }
            }
            None => Err(ArgsError::MissingFilePath),
        }
    }

    pub fn new() -> Self {
        Args {
            file_path: None,
            nails: unsafe { NonZero::new_unchecked(512) },
            nail_kind: NailShape::Circular(1.0),
            resolution: unsafe { NonZero::new_unchecked(1000) },
            precision: Precision::Single,
            darkness_mode: DarknessMode::Percentage(0.93),
            min_nail_distance: 20,
            threads: 4000,
            colors: vec![NamedColor {
                name: String::from("Black"),
                color: [0, 0, 0],
            }],
            groups: vec![ColorGroupSettings::new(vec![0], 0.5)],
            tickness: 1.0
        }
    }
}

#[derive(Debug, Error)]
pub enum ArgsError {
    #[error("Missing file path.")]
    MissingFilePath,

    #[error(transparent)]
    AlgorithmError(string_art::Error),

    #[error("Unimplemented feature: {0}.")]
    UnimplementedFeature(&'static str),

    #[error(transparent)]
    IOError(std::io::Error),

    #[error(transparent)]
    ImageError(image::ImageError),
}

#[derive(Debug, Clone)]
pub struct NamedColor {
    pub name: String,
    pub color: [u8; 3],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DarknessMode {
    Flat(f32),
    Percentage(f32),
}

impl From<DarknessMode> for WidgetText {
    fn from(value: DarknessMode) -> Self {
        WidgetText::RichText(RichText::new(match value {
            DarknessMode::Flat(_) => "Flat",
            DarknessMode::Percentage(_) => "Percentage",
        }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NailShape {
    Circular(f32),
    Point,
}

impl From<NailShape> for WidgetText {
    fn from(value: NailShape) -> Self {
        WidgetText::RichText(RichText::new(match value {
            NailShape::Circular(_) => "Circular",
            NailShape::Point => "Point",
        }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    Single,
    Double,
}