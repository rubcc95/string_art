use std::{path::Path, str::FromStr};

use clap::{Parser, ValueEnum};
use num_traits::AsPrimitive;
use palette::{FromColor, Srgb};
use string_art::{
    nails::{self, Handle},
    ColorMapSettings, Darkness, FlatDarkness, Float, Lab, PercentageDarkness, StringArt, Table,
};
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file path.
    #[arg()]
    input: String,

    /// Number of nails surrounding the image.
    #[arg(short, long, default_value_t = 512)]
    nails: usize,

    #[arg(long, default_value_t = NailKind::Circular)]
    nail_kind: NailKind,
    /// Size in pixels of the longest side of the image.
    #[arg(short, long, default_value_t = 1024)]
    resolution: u32,

    /// Precision of calculations (Single/Double).
    #[arg(short, long, default_value_t = Precision::Single)]
    precision: Precision,

    /// Darkness value modifier.
    #[arg(long)]
    darkness_value: Option<f32>,

    /// Darkness mode of processing.
    #[arg(long, default_value_t = DarknessMode::Percentage)]
    darkness_mode: DarknessMode,

    /// Minimum nail count between linked nails.
    #[arg(long)]
    min_nail_distance: Option<usize>,

    #[arg(long, short, default_value_t = 8000)]
    threads: usize,

    #[arg(long, short)]
    interval: Option<usize>,

    #[arg(long, default_value_t = 1.0)]
    radius: f32,

    #[arg(long, short)]
    colors: Vec<NamedColor>,
}

#[derive(Debug, Clone)]
struct NamedColor {
    name: String,
    color: Option<Srgb<u8>>,
}

struct NamedColorRef<'a> {
    name: &'a str,
    color: Option<&'a Srgb<u8>>,
}

impl FromStr for NamedColor {
    type Err = NamedColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Formato esperado: "name:l,a,b" o "name:#FFFFFF"
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            1 => Ok(Self {
                name: parts[0].to_string(),
                color: None,
            }),
            2 => {
                let color_str = parts[1];
                if let Some(color) = parse_hex_color(color_str)? {
                    Ok(NamedColor {
                        name: parts[0].to_string(),
                        color: Some(color),
                    })
                } else {
                    let rgb: Vec<&str> = color_str.split(',').collect();
                    if rgb.len() != 3 {
                        return Err(NamedColorParseError::InvalidRgb);
                    }
                    Ok(NamedColor {
                        name: parts[0].to_string(),
                        color: Some(Srgb::<u8>::new(
                            rgb[0]
                                .parse::<u8>()
                                .map_err(|_| NamedColorParseError::InvalidRgb)?,
                            rgb[1]
                                .parse::<u8>()
                                .map_err(|_| NamedColorParseError::InvalidRgb)?,
                            rgb[2]
                                .parse::<u8>()
                                .map_err(|_| NamedColorParseError::InvalidRgb)?,
                        )),
                    })
                }
            }
            _ => Err(NamedColorParseError::InvalidFormat),
        }
    }
}

fn parse_hex_color(s: &str) -> Result<Option<Srgb<u8>>, NamedColorParseError> {
    let s = s.trim();
    let s = s
        .strip_prefix('#')
        .or_else(|| s.strip_prefix("0x"))
        .unwrap_or(s);

    match s.len() {
        6 => {
            let r =
                u8::from_str_radix(&s[0..2], 16).map_err(|_| NamedColorParseError::InvalidHex)?;
            let g =
                u8::from_str_radix(&s[2..4], 16).map_err(|_| NamedColorParseError::InvalidHex)?;
            let b =
                u8::from_str_radix(&s[4..6], 16).map_err(|_| NamedColorParseError::InvalidHex)?;
            Ok(Some(Srgb::new(r, g, b)))
        }
        3 => {
            let r = u8::from_str_radix(&s[0..1].repeat(2), 16)
                .map_err(|_| NamedColorParseError::InvalidHex)?;
            let g = u8::from_str_radix(&s[1..2].repeat(2), 16)
                .map_err(|_| NamedColorParseError::InvalidHex)?;
            let b = u8::from_str_radix(&s[2..3].repeat(2), 16)
                .map_err(|_| NamedColorParseError::InvalidHex)?;
            Ok(Some(Srgb::new(r, g, b)))
        }
        _ => Ok(None),
    }
}

#[derive(Debug, Error)]
enum NamedColorParseError {
    #[error("Formato inválido. Usa 'nombre:color'")]
    InvalidFormat,
    #[error("Formato RGB inválido. Usa valores numéricos separados por comas.")]
    InvalidRgb,
    #[error("Formato hexadecimal inválido. Usa #RRGGBB, 0xRRGGBB o RRGGBB.")]
    InvalidHex,
}

#[derive(Clone, Copy, Debug)]
enum DarknessMode {
    Flat,
    Percentage,
}

impl ValueEnum for DarknessMode {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Flat, Self::Percentage]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Flat => clap::builder::PossibleValue::new("Flat")
                .alias("flat")
                .alias("Flatten")
                .alias("flatten"),
            Self::Percentage => clap::builder::PossibleValue::new("Percentage")
                .alias("percentage")
                .alias("per")
                .alias("Per"),
        })
    }
}

impl ToString for DarknessMode {
    fn to_string(&self) -> String {
        match self {
            Self::Flat => String::from("Flat"),
            Self::Percentage => String::from("Percentage"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum NailKind {
    Circular,
    Point,
}

impl ValueEnum for NailKind {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Circular, Self::Point]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Circular => clap::builder::PossibleValue::new("Circular").alias("circular"),
            Self::Point => clap::builder::PossibleValue::new("Point").alias("point"),
        })
    }
}

impl ToString for NailKind {
    fn to_string(&self) -> String {
        match self {
            Self::Circular => String::from("Circular"),
            Self::Point => String::from("Point"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Precision {
    Single,
    Double,
}

impl ValueEnum for Precision {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Single, Self::Double]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Single => clap::builder::PossibleValue::new("Single")
                .alias("single")
                .alias("f32"),
            Self::Double => clap::builder::PossibleValue::new("Double")
                .alias("double")
                .alias("f64"),
        })
    }
}

impl ToString for Precision {
    fn to_string(&self) -> String {
        match self {
            Self::Single => String::from("Single"),
            Self::Double => String::from("Double"),
        }
    }
}

fn main() {
    let args = Args::parse();
    match args.precision {
        Precision::Single => with_precision::<f32>(&args),
        Precision::Double => with_precision::<f64>(&args),
    }
}

fn with_precision<S: Float + Sync + Send>(args: &Args)
where
    f32: AsPrimitive<S>,
    usize: AsPrimitive<S>,
    u8: AsPrimitive<S>,
{
    match args.darkness_mode {
        DarknessMode::Flat => with_darkness_mode::<S, FlatDarkness<S>>(
            args,
            FlatDarkness(args.darkness_value.unwrap_or(5.0).as_()),
        ),
        DarknessMode::Percentage => with_darkness_mode::<S, PercentageDarkness<S>>(
            args,
            PercentageDarkness(args.darkness_value.unwrap_or(0.93).as_()),
        ),
    }
}

fn with_darkness_mode<S: Float + Sync + Send, D: Darkness<S>>(args: &Args, darkness: D)
where
    f32: AsPrimitive<S>,
    f32: AsPrimitive<S>,
    usize: AsPrimitive<S>,
    u8: AsPrimitive<S>,
{
    match args.nail_kind {
        NailKind::Circular => {
            let a = nails::Circular::new(args.radius.as_());
            with_nail_kind(args, darkness, a);
        }
        NailKind::Point => todo!(),
    }
}

fn with_nail_kind<
    S: Float + Sync + Send,
    D: Darkness<S>,
    N: nails::Builder<Scalar = S, Handle: Sync + Send + Handle<Link: Default + Sync + Send>>,
>(
    args: &Args,
    darkness: D,
    nail_builder: N,
) where
    usize: AsPrimitive<S>,
    u8: AsPrimitive<S>,
{
    let table = Table::ellipse(
        image::open(args.input.clone()).unwrap().resize(
            args.resolution,
            args.resolution,
            image::imageops::FilterType::Lanczos3,
        ),
        nail_builder,
        args.nails,
    );
    let min_nail_distance = args.min_nail_distance.unwrap_or(args.nails / 10);
    let mut builder = if args.colors.is_empty() {
        StringArt::new(
            table,
            core::iter::once(ColorMapSettings::new(
                "Black",
                Lab::new(S::ZERO, S::ZERO, S::ZERO),
                0,
                <N::Handle as Handle>::Link::default(),
            )),
            min_nail_distance,
            darkness,
        )
    } else {
        StringArt::new(
            table,
            args.colors.iter().map(|color| {
                let rgb = color
                    .color
                    .unwrap_or_else(|| match color.name.to_lowercase().as_str() {
                        "red" => Srgb::new(255, 0, 0),
                        "green" => Srgb::new(0, 255, 0),
                        "blue" => Srgb::new(0, 0, 255),
                        "yellow" => Srgb::new(255, 255, 0),
                        "black" => Srgb::new(0, 0, 0),
                        "white" => Srgb::new(255, 255, 255),
                        "gray" | "grey" => Srgb::new(128, 128, 128),
                        "orange" => Srgb::new(255, 165, 0),
                        "purple" => Srgb::new(128, 0, 128),
                        "brown" => Srgb::new(165, 42, 42),
                        "pink" => Srgb::new(255, 192, 203),
                        "cyan" => Srgb::new(0, 255, 255),
                        "magenta" => Srgb::new(255, 0, 255),
                        "lime" => Srgb::new(50, 205, 50),
                        "teal" => Srgb::new(0, 128, 128),
                        "navy" => Srgb::new(0, 0, 128),
                        "indigo" => Srgb::new(75, 0, 130),
                        "violet" => Srgb::new(238, 130, 238),
                        "gold" => Srgb::new(255, 215, 0),
                        "silver" => Srgb::new(192, 192, 192),
                        "beige" => Srgb::new(245, 245, 220),
                        "ivory" => Srgb::new(255, 255, 240),
                        "peach" => Srgb::new(255, 218, 185),
                        "chocolate" => Srgb::new(210, 105, 30),
                        _ => panic!("Color '{}' is not valid", color.name),
                    });
                ColorMapSettings::new(
                    color.name.as_str(),
                    Lab::from_color(Srgb::new(
                        rgb.red.as_() * S::TWO_FIVE_FIVE,
                        rgb.green.as_() * S::TWO_FIVE_FIVE,
                        rgb.blue.as_() * S::TWO_FIVE_FIVE,
                    )),
                    0,
                    <N::Handle as Handle>::Link::default(),
                )
            }),
            min_nail_distance,
            darkness,
        )
    };

    let path = Path::new(&args.input);
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Invalid file name");
    let out_folder = Path::new(&args.input)
        .parent()
        .unwrap_or(Path::new("."))
        .join("output");
    std::fs::create_dir_all(out_folder.as_path()).expect("Output directory can not be created");

    if let Some(step) = args.interval {
        let mut iteration = 1;
        let mut current = step;
        while current < args.threads {
            builder.compute(current);
            builder
                .save_image_svg(
                    out_folder.join(format!("{}_{}.svg", file_name, iteration)),
                    1.0,
                )
                .expect("Failed creating svg image");
            current += step;
            iteration += 1;
        }
    }
    builder.compute(args.threads);
    builder
        .save_image_svg(out_folder.join(format!("{}.svg", file_name)), 0.5)
        .expect("Failed creating svg image");
}
