use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use string_art::{
    Board, ValidPipelineLayer, board::ValidBoard, geometry::Segment, math::Frac16, nails,
};
use wasm_bindgen::prelude::*;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Step {
    pub color: u32,
    pub segment: Segment<f32>,
    pub nail: usize,
    pub link: u8,
}

// #[wasm_bindgen]
// #[derive(Copy, Clone, Serialize, Deserialize)]
// pub struct Segment(geometry::Segment<f32>);

// #[wasm_bindgen]
// impl Segment {
//     #[wasm_bindgen(getter)]
//     pub fn start(&self) -> Point {
//         Point(self.0.start)
//     }

//     #[wasm_bindgen(getter)]
//     pub fn end(&self) -> Point {
//         Point(self.0.end)
//     }
// }

// #[wasm_bindgen]
// #[derive(Copy, Clone, Serialize, Deserialize)]
// pub struct Point(geometry::Point<f32>);

// #[wasm_bindgen]
// impl Point {
//     #[wasm_bindgen(getter)]
//     pub fn x(&self) -> f32 {
//         self.0.x
//     }

//     #[wasm_bindgen(getter)]
//     pub fn y(&self) -> f32 {
//         self.0.y
//     }
// }

#[wasm_bindgen]
pub struct Computation(Box<dyn Iterator<Item = JsValue>>);

#[wasm_bindgen]
impl Computation {
    #[wasm_bindgen(constructor)]
    pub fn new(settings: JsValue) -> Result<Self, WasmFailErrorError> {
        let settings = Settings::new(settings)?;
        let pipeline = string_art::Monocolor::from_image(
            &image::load_from_memory(&settings.buffer).to_wasm()?,
        );
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
                Frac16::from_bits((u16::MAX as f32 * settings.decay) as u16),
            ),
        ))))
    }

    pub fn next(&mut self) -> Option<JsValue> {
        self.0.next()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug)]
pub struct WasmFailErrorError {
    pub message: String,
}

impl core::fmt::Display for WasmFailErrorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WasmFailErrorError {}

pub trait ResultExt {
    type Ok;

    fn to_wasm(self) -> Result<Self::Ok, WasmFailErrorError>;
}

impl<T, E: core::fmt::Display> ResultExt for Result<T, E> {
    type Ok = T;

    fn to_wasm(self) -> Result<T, WasmFailErrorError> {
        self.map_err(|e| WasmFailErrorError {
            message: e.to_string(),
        })
    }
}

struct ComputationImpl<B: Board, P: string_art::Pipeline>(string_art::Computation<B, P>);

impl<B: WasmBoard, P: WasmPipeline<B::Anchor>> Iterator for ComputationImpl<B, P> {
    type Item = JsValue;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|step| {
            {
                to_value(&Step {
                    color: step.layer.to_color(),
                    segment: step.segment,
                    nail: step.anchor.anchor_idx(),
                    link: step.anchor.link_idx(),
                })
            }
            .unwrap()
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

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub decay: f32,
    #[serde(rename = "minNailDistance")]
    min_nail_distance: usize,
    #[serde(rename = "nailCount")]
    nail_count: usize,
    #[serde(rename = "circularNailRadius")]
    circular_nail_radius: f32,
    buffer: Vec<u8>,
}

impl Settings {
    pub fn new(js_obj: JsValue) -> Result<Self, WasmFailErrorError> {
        from_value(js_obj).to_wasm()
    }
}
