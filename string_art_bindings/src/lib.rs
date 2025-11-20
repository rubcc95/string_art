use string_art::{Board, ValidPipelineLayer, board::ValidBoard, geometry, math::Frac16, nails};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Step {
    pub color: u32,
    pub segment: Segment,
    pub nail: usize,
    pub link: u8,
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Segment(geometry::Segment<f32>);

#[wasm_bindgen]
impl Segment {
    #[wasm_bindgen(getter)]
    pub fn start(&self) -> Point {
        Point(self.0.start)
    }

    #[wasm_bindgen(getter)]
    pub fn end(&self) -> Point {
        Point(self.0.end)
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Point(geometry::Point<f32>);

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> f32 {
        self.0.x
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> f32 {
        self.0.y
    }
}

#[wasm_bindgen]
pub struct Computation(Box<dyn Iterator<Item = Step>>);

impl Computation {
    pub fn new<B: WasmBoard, P: WasmPipeline<B::Anchor>>(
        settings: &Settings,
        image_buffer: &[u8],
        decay: f32,
    ) -> Result<Self, WasmError> {
        let pipeline =
            string_art::Monocolor::from_image(&image::load_from_memory(image_buffer).to_wasm()?);
        let board = string_art::ellipse::Ellipse::new(
            pipeline.rect().as_(),
            string_art::nails::UniformCircular(settings.circular_nail_radius),
            settings.nail_count,
            settings.min_nail_distance,
        )
        .to_wasm()?;
        Ok(Self(Box::new(ComputationImpl(
            string_art::computation::Computation::new(
                pipeline,
                board,
                Frac16::from_bits((u16::MAX as f32 * decay) as u16),
            ),
        ))))
    }
}
#[wasm_bindgen]
impl Computation {
    pub fn next(&mut self) -> Option<Step> {
        self.0.next()
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmError {
    pub message: String,
}

pub trait ResultExt {
    type Ok;

    fn to_wasm(self) -> Result<Self::Ok, WasmError>;
}

impl<T, E: core::fmt::Display> ResultExt for Result<T, E> {
    type Ok = T;

    fn to_wasm(self) -> Result<T, WasmError> {
        self.map_err(|e| WasmError {
            message: e.to_string(),
        })
    }
}

struct ComputationImpl<B: Board, P: string_art::Pipeline>(string_art::Computation<B, P>);

impl<B: WasmBoard, P: WasmPipeline<B::Anchor>> Iterator for ComputationImpl<B, P> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|step| Step {
            color: step.layer.to_color(),
            segment: Segment(step.segment),
            nail: step.anchor.anchor_idx(),
            link: step.anchor.link_idx(),
        })
    }
}

pub trait WasmBoard: ValidBoard<Anchor: WasmAnchor> + 'static {}

impl<T: ValidBoard<Anchor: WasmAnchor> + 'static> WasmBoard for T {}

pub trait WasmPipeline<A>
where
    for<'a> Self: string_art::Pipeline<Layer<'a, A>: ValidPipelineLayer>,
    Self: string_art::Pipeline<MapId: WasmMapId, Weight = Frac16> + 'static,
{
}

impl<P, A> WasmPipeline<A> for P
where
    for<'a> P: string_art::Pipeline<Layer<'a, A>: ValidPipelineLayer>,
    P: string_art::Pipeline<MapId: WasmMapId, Weight = Frac16> + 'static,
{
}

pub trait WasmMapId {
    fn to_color(&self) -> u32;
}

impl WasmMapId for () {
    fn to_color(&self) -> u32 {
        0
    }
}

pub trait WasmAnchor {
    fn anchor_idx(&self) -> usize;

    fn link_idx(&self) -> u8;
}

impl<L: WasmLink> WasmAnchor for nails::Anchor<L> {
    fn anchor_idx(&self) -> usize {
        self.idx
    }

    fn link_idx(&self) -> u8 {
        self.link.to_u8()
    }
}

trait WasmLink {
    fn to_u8(&self) -> u8;
}

impl WasmLink for nails::circular::Direction {
    fn to_u8(&self) -> u8 {
        match *self {
            Self::CLOCK_WISE => 0,
            Self::COUNTER_CLOCK_WISE => 1,
        }
    }
}

impl WasmLink for nails::point::Link {
    fn to_u8(&self) -> u8 {
        0
    }
}

#[wasm_bindgen]
pub struct Settings {
    pub decay: f32,
    min_nail_distance: usize,
    nail_count: usize,
    circular_nail_radius: f32,
}

#[wasm_bindgen]
impl Settings {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            decay: 0.1,
            min_nail_distance: 20,
            nail_count: 512,
            circular_nail_radius: 0.2,
        }
    }

    #[wasm_bindgen(getter, js_name = minNailDistance)]
    pub fn min_nail_distance(&self) -> usize {
        self.min_nail_distance
    }

    #[wasm_bindgen(setter, js_name = minNailDistance)]
    pub fn set_min_nail_distance(&mut self, distance: usize) {
        self.min_nail_distance = distance;
    }

    #[wasm_bindgen(getter, js_name = nailCount)]
    pub fn nail_count(&self) -> usize {
        self.nail_count
    }

    #[wasm_bindgen(setter, js_name = nailCount)]
    pub fn set_nail_count(&mut self, count: usize) {
        self.nail_count = count;
    }

    #[wasm_bindgen(getter, js_name = circularNailRadius)]
    pub fn circular_nail_radius(&self) -> f32 {
        self.circular_nail_radius
    }

    #[wasm_bindgen(setter, js_name = circularNailRadius)]
    pub fn set_circular_nail_radius(&mut self, radius: f32) {
        self.circular_nail_radius = radius;
    }
}
