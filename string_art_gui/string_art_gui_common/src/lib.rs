use board::IntoBoard;
use derive_more::*;
use ellipse::EllipseBoard;
use geometry::*;
use image::GenericImageView;
use math::*;
use std::fmt::{Display, Formatter, Write};
use string_art::{Color as _, *};

pub mod input;
pub use input::Input;

pub const BORDER: usize = 30;
pub const F_BORDER: f32 = BORDER as f32;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Output {
    Init(Data),
    Steps(Vec<Step>),
    Err(String),
    Done(Vec<Step>, Vec<Anchor>),
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Data {
    pub svg: Svg,
    pub estimated_steps: usize,
    pub colors: Vec<Color>,
}

impl Data {
    pub fn append_steps(
        &mut self,
        steps: &mut Vec<Step>,
        mut new_steps: Vec<Step>,
    ) -> Result<(), String> {
        let mut new_svg = String::new();

        for step in new_steps.iter().rev() {
            let segment = step.segment;
            let color = &self.colors[step.color as usize];
            write!(
            &mut new_svg,
            "<line opacity=\"1\" stroke=\"rgb({},{},{})\" stroke-width=\"0.2500\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/>",
            color.r, color.g, color.b,
            segment.start.x + F_BORDER, segment.end.x + F_BORDER,
            segment.start.y + F_BORDER, segment.end.y + F_BORDER,
        ).map_err(|e| e.to_string())?;
        }

        let old_svg = std::mem::replace::<String>(&mut self.svg, new_svg);
        self.svg.push_str(&old_svg);
        steps.append(&mut new_steps);

        Ok(())
    }
}
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub name: String,
}

#[derive(Clone, Deref, DerefMut)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Step {
    pub segment: Segment<f32>,
    #[deref]
    #[deref_mut]
    pub anchor: Anchor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Anchor {
    pub index: usize,
    pub link: u8,
    pub color: u8,
}

#[derive(Clone, Deref, DerefMut)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Svg {
    view_box_rect: Rect<usize>,
    #[deref]
    #[deref_mut]
    content: String,
}

impl Display for Svg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<svg viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">{}</svg>",
            self.view_box_rect.width + 2 * BORDER,
            self.view_box_rect.height + 2 * BORDER,
            self.content
        )
    }
}

// ===================
// COMPATIBILITY TRAITS
// ===================

pub trait CompatibleComputation: Iterator<Item = Result<Step, String>> {
    fn init(&self) -> Result<Data, String>;

    fn start_anchors(&self) -> Vec<Anchor>;
}

trait CompatibleBoard: Board<Anchor: CompatibleAnchor> {
    fn init_svg(&self, data: &mut Svg);
}

trait CompatibleHandle: nails::Handle {
    fn init_svg(&self, target: &mut Svg, nail: &Self::Nail, index: usize);
}

trait CompatibleAnchor {
    fn index(&self) -> usize;
    fn link(&self) -> u8;
}

trait CompatibleIndex {
    fn to_idx(&self) -> u8;
    fn from_idx(index: u8) -> Self
    where
        Self: Sized;
}

trait CompatibleLink: CompatibleIndex + Display {}

impl<T: CompatibleIndex + Display> CompatibleLink for T {}

impl<N> CompatibleBoard for EllipseBoard<N>
where
    N: nails::Builder<Handle: CompatibleHandle> + nails::Builder<Link: CompatibleLink>,
{
    fn init_svg(&self, data: &mut Svg) {
        for (index, nail) in self.nails.iter().enumerate() {
            self.handle.init_svg(data, nail, index);
        }
    }
}

fn write_nail_on_svg(target: &mut Svg, pos: Point<f32>, radius: f32, index: usize){

    write!(
        target,
        "<circle cx=\"{}\" cy=\"{}\" fill=\"black\" r=\"{radius}\"/>",
        pos.x + F_BORDER,
        pos.y + F_BORDER,
    )
    .unwrap();

    if index % 8 == 0{
        let center = Point::from(target.view_box_rect).as_::<f32>() / 2.0;
        let text_pos = pos + (pos - center).normalize() * 15.0;
        write!(
            target,
            "<text x=\"{}\" y=\"{}\" font-family=\"Arial\" font-size=\"10\" fill=\"black\" text-anchor=\"middle\" dominant-baseline=\"middle\">
                {index}
            </text>",
            text_pos.x + F_BORDER,
            text_pos.y + F_BORDER,
        )
        .unwrap();
    }
}

impl CompatibleHandle for nails::UniformCircular {
    fn init_svg(&self, target: &mut Svg, nail: &Self::Nail, index: usize) {
        write_nail_on_svg(target, *nail, self.0, index);
    }
}

impl CompatibleHandle for nails::Point {
    fn init_svg(&self, target: &mut Svg, nail: &Self::Nail, index: usize) {
        write_nail_on_svg(target, *nail, 0.1, index);
    }
}

impl<L: CompatibleLink> CompatibleAnchor for nails::Anchor<L> {
    fn index(&self) -> usize {
        self.idx
    }
    fn link(&self) -> u8 {
        self.link.to_idx()
    }
}

impl CompatibleIndex for nails::circular::Direction {
    fn to_idx(&self) -> u8 {
        match **self {
            circle::Direction::ClockWise => 0,
            circle::Direction::CounterClockWise => 1,
        }
    }
    fn from_idx(index: u8) -> Self {
        match index {
            0 => Self::CLOCK_WISE,
            1 => Self::COUNTER_CLOCK_WISE,
            _ => unreachable!(),
        }
    }
}

impl CompatibleIndex for nails::point::Link {
    fn to_idx(&self) -> u8 {
        0
    }
    fn from_idx(_: u8) -> Self {
        Self
    }
}

impl CompatibleIndex for monocolor::MonocolorIndex {
    fn to_idx(&self) -> u8 {
        0
    }
    fn from_idx(index: u8) -> Self {
        match index {
            0 => Self,
            _ => unreachable!(),
        }
    }
}

// ===================
// COMPUTATION WRAPPER
// ===================
struct Wrapper<B, P: computation::Pipeline, D> {
    cmpt: string_art::Computation<B, P, D>,
}

impl<B, P, D> Iterator for Wrapper<B, P, D>
where
    B: CompatibleBoard,
    P: computation::Pipeline<Anchor = B::Anchor, Index: CompatibleIndex>,
    D: decay::Fn<<P::Scalar as Scalar>::Frac>,
{
    type Item = Result<Step, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.cmpt.next().map(|next| match next {
            Ok(step) => Ok(Step {
                segment: step.segment,
                anchor: Anchor {
                    index: step.anchor.index(),
                    link: step.anchor.link(),
                    color: step.color.to_idx(),
                },
            }),
            Err(err) => Err(err.to_string()),
        })
    }
}

impl<B, P, D> CompatibleComputation for Wrapper<B, P, D>
where
    B: CompatibleBoard,
    P: computation::Pipeline<Anchor = B::Anchor, Index: CompatibleIndex>
        + computation::Pipeline<Color: Display + string_art::Color<Unit: Into<u8>>>,
    D: decay::Fn<<P::Scalar as Scalar>::Frac>,
{
    fn init(&self) -> Result<Data, String> {
        let mut svg = Svg {
            view_box_rect: *self.cmpt.rect(),
            content: String::new(),
        };
        self.cmpt.board().init_svg(&mut svg);
        let colors = self
            .cmpt
            .pipeline()
            .color_maps()
            .iter()
            .map(|map| {
                let color = &map.color;
                Color {
                    r: color.r().into(),
                    g: color.g().into(),
                    b: color.b().into(),
                    name: color.to_string(),
                }
            })
            .collect();
        Ok(Data {
            svg,
            estimated_steps: self.cmpt.pipeline().size_hint().0,
            colors,
        })
    }

    fn start_anchors(&self) -> Vec<Anchor> {
        self.cmpt
            .pipeline()
            .color_maps()
            .into_iter()
            .enumerate()
            .map(|(index, a)| {
                let anchor = &a.anchor;
                Anchor {
                    index: anchor.index(),
                    link: anchor.link(),
                    color: index as u8,
                }
            })
            .collect()
    }
}

// ===================
// COMPUTATION FUNCS
// ===================

pub fn computation(
    input: Input,
) -> Result<Box<dyn CompatibleComputation>, Box<dyn std::error::Error>> {
    if input::pipeline::Mode::Multi == input.pipeline {
        return Err(Box::new(UnimplementedError("Multicolor pipeline")));
    }
    let (mut width, mut height) = (input.image.width(), input.image.height());

    let resolution = input.computation_settings.resolution;
    if resolution != height || resolution != width {
        use std::cmp::max;

        (width, height) = {
            let resolution = f64::from(resolution);
            let wratio = resolution / f64::from(width);
            let hratio = resolution / f64::from(height);
            let ratio = f64::min(wratio, hratio);

            let nw = max((f64::from(width) * ratio).round() as u64, 1);
            let nh = max((f64::from(height) * ratio).round() as u64, 1);

            if nw > u64::from(u32::MAX) {
                let ratio = f64::from(u32::MAX) / f64::from(width);
                (u32::MAX, max((f64::from(height) * ratio).round() as u32, 1))
            } else if nh > u64::from(u32::MAX) {
                let ratio = f64::from(u32::MAX) / f64::from(height);
                (max((f64::from(width) * ratio).round() as u32, 1), u32::MAX)
            } else {
                (nw as u32, nh as u32)
            }
        };
    }

    let image = image::imageops::resize(
        &*input.image,
        width,
        height,
        image::imageops::FilterType::Lanczos3,
    );

    match input.nail_shape {
        input::NailShape::Circular(rad) => with_nails(image, nails::UniformCircular(rad), input),
        input::NailShape::Point => with_nails(image, nails::Point, input),
        input::NailShape::Hook(_) => Err(Box::new(UnimplementedError("Hook nail shape"))),
    }
}

fn with_nails<'a, N>(
    image: impl GenericImageView<Pixel: image::Pixel<Subpixel = u8>>,
    nails: N,
    input: Input,
) -> Result<Box<dyn CompatibleComputation + 'a>, Box<dyn std::error::Error + 'a>>
where
    usize: TryInto<N::Link, Error: std::error::Error + 'static>,
    N: 'a + nails::Builder<Handle: CompatibleHandle> + nails::Builder<Link: CompatibleLink>,
{
    match input.board_shape {
        input::BoardShape::Ellipse(ellipse) => {
            let board = string_art::Ellipse::new(
                Rect::new(image.height() as f32, image.width() as f32),
                nails,
                ellipse.nail_count,
                ellipse.min_nail_distance,
            )
            .map_err(|err| Box::new(err))?;
            with_board(image, board, input)
        }
        input::BoardShape::Rectangle(_) => {
            Err(Box::new(UnimplementedError("Rectangle board shape")))
        }
    }
}

fn with_board<'a, B>(
    image: impl GenericImageView<Pixel: image::Pixel<Subpixel = u8>>,
    board: B,
    input: Input,
) -> Result<Box<dyn CompatibleComputation + 'a>, Box<dyn std::error::Error + 'a>>
where
    B: IntoBoard<Board: CompatibleBoard + 'a>,
{
    match input.computation_settings.decay_fn {
        input::computation_settings::DecayFn::Flat(flat) => {
            with_decay(image, board, decay::Flat(flat.to_fixed()), input)
        }
        input::computation_settings::DecayFn::Percentage(per) => {
            with_decay(image, board, decay::Percentage(per.to_fixed()), input)
        }
    }
}

fn with_decay<'a, B, F>(
    image: impl GenericImageView<Pixel: image::Pixel<Subpixel = u8>>,
    board: B,
    decay: F,
    input: Input,
) -> Result<Box<dyn CompatibleComputation + 'a>, Box<dyn std::error::Error + 'a>>
where
    B: IntoBoard<Board: CompatibleBoard + 'a>,
    F: decay::Fn<math::Frac32> + 'a,
{
    Ok(Box::new(Wrapper {
        cmpt: string_art::Computation::new(
            Monocolor::<_, Scalar32>::from_image(
                &image,
                match input.pipeline {
                    input::pipeline::Mode::Mono(threads) => threads,
                    input::pipeline::Mode::Multi => 5000,
                },
            ),
            board,
            Rect::new(image.height() as usize, image.width() as usize),
            decay,
        )
        .map_err(|err| Box::new(err))?,
    }))
}

#[derive(Debug, thiserror::Error)]
#[error("{0} feature is not available yet. It will be added in a future update.")]
struct UnimplementedError(&'static str);
