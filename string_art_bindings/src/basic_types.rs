use string_art::{
    Board, Pipeline, ValidPipelineLayer, board::ValidBoard, geometry, math::Frac16, nails,
};
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
        board: B,
        pipeline: P,
        decay: f32,
    ) -> Self {
        Self(Box::new(ComputationImpl(
            string_art::computation::Computation::new(
                pipeline,
                board,
                Frac16::from_bits((u16::MAX as f32 * decay) as u16),
            ),
        )))
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

struct ComputationImpl<B: Board, P: Pipeline>(string_art::Computation<B, P>);

impl<B, P> Iterator for ComputationImpl<B, P>
where
    B: ValidBoard,
    for<'a> P: Pipeline<Layer<'a, B::Anchor>: ValidPipelineLayer>,
    P: Pipeline<MapId: WasmMapId>,
    B: Board<Anchor: WasmAnchor>,
{
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
    for<'a> Self: Pipeline<Layer<'a, A>: ValidPipelineLayer>,
    Self: Pipeline<MapId: WasmMapId, Weight = Frac16> + 'static,
{
}

impl<P, A> WasmPipeline<A> for P
where
    for<'a> P: Pipeline<Layer<'a, A>: ValidPipelineLayer>,
    P: Pipeline<MapId: WasmMapId, Weight = Frac16> + 'static,
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
