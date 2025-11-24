use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use string_art::{
    Board, Color32, ValidPipelineLayer,
    board::ValidBoard,
    geometry::{Circle, Rect, Segment},
    math::Frac16,
    nails,
};
use wasm_bindgen::prelude::*;
use web_sys::js_sys::Function;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Step {
    pub color: u32,
    pub segment: Segment<f32>,
    pub nail: usize,
    pub link: u8,
}
#[wasm_bindgen]
pub struct Computation(Box<dyn ComputationImpl>);

#[wasm_bindgen]
impl Computation {
    #[wasm_bindgen(constructor)]
    pub fn new(settings: JsValue) -> Result<Self, WasmError> {
        let settings = Settings::new(settings)?;
        let pipeline = string_art::Monocolor::from_image(
            &image::load_from_memory(&settings.buffer).to_wasm()?,
        );
        let ellipse = string_art::Ellipse::new(
            pipeline.rect().as_(),
            string_art::nails::UniformCircular(settings.circular_nail_radius),
            settings.nail_count,
        );

        let board =
            string_art::ellipse::Board::new(&ellipse, settings.min_nail_distance).to_wasm()?;
        Ok(Self(Box::new(ComputationWrapper(
            string_art::computation::Computation::new(
                pipeline,
                board,
                Frac16::from_bits((u16::MAX as f32 * settings.decay) as u16),
            ),
        ))))
    }

    pub fn rect(&self) -> Result<JsValue, WasmError> {
        to_value(&self.0.rect()).to_wasm()
    }

    pub fn next(&mut self) -> Option<JsValue> {
        self.0.next()
    }
}

trait ComputationImpl: Iterator<Item = JsValue> {
    fn rect(&self) -> Rect<u32>;
}

struct ComputationWrapper<B: Board, P: string_art::Pipeline>(string_art::Computation<B, P>);

impl<B: WasmBoard, P: WasmPipeline<B::Anchor>> ComputationImpl for ComputationWrapper<B, P> {
    fn rect(&self) -> Rect<u32> {
        self.0.rect()
    }
}

impl<B: WasmBoard, P: WasmPipeline<B::Anchor>> Iterator for ComputationWrapper<B, P> {
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
    pub min_nail_distance: usize,
    #[serde(rename = "nailCount")]
    pub nail_count: usize,
    #[serde(rename = "circularNailRadius")]
    pub circular_nail_radius: f32,
    pub buffer: Vec<u8>,
}

impl Settings {
    pub fn new(js_obj: JsValue) -> Result<Self, WasmError> {
        from_value(js_obj).to_wasm()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug)]
pub struct WasmError {
    pub message: String,
}

impl core::fmt::Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WasmError {}

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

#[wasm_bindgen]
pub struct DrawBackend {
    draw_circle: JsValue,
    draw_circunference: JsValue,
    draw_segment: JsValue,
}

#[wasm_bindgen]
impl DrawBackend {
    #[wasm_bindgen(constructor)]
    pub fn new(draw_circle: JsValue, draw_circunference: JsValue, draw_segment: JsValue) -> Self {
        Self {
            draw_circle,
            draw_circunference,
            draw_segment,
        }
    }
}

impl DrawBackend {
    fn call_js(&self, f: &JsValue, args: &[JsValue]) {
        let func: &Function = f.dyn_ref().expect("Expected a JS function");
        func.apply(
            &JsValue::NULL,
            &web_sys::js_sys::Array::from_iter(args.iter()),
        )
        .expect("JS call failed");
    }
}

impl string_art::DrawBackend for DrawBackend {
    fn draw_circle(&mut self, circle: Circle<f32>, color: impl Color32) {
        self.call_js(
            &self.draw_circle,
            &[
                to_value(&circle.center).unwrap(),
                to_value(&circle.radius).unwrap(),
                to_value(&color_to_int(&color)).unwrap(),
            ],
        );
    }

    fn draw_circunference(&mut self, circle: Circle<f32>, stroke: f32, color: impl Color32) {
        self.call_js(
            &self.draw_circunference,
            &[
                to_value(&circle.center).unwrap(),
                to_value(&circle.radius).unwrap(),
                to_value(&stroke).unwrap(),
                to_value(&color_to_int(&color)).unwrap(),
            ],
        );
    }

    fn draw_segment(&mut self, segment: Segment<f32>, stroke: f32, color: impl Color32) {
        self.call_js(
            &self.draw_segment,
            &[
                to_value(&segment).unwrap(),
                to_value(&stroke).unwrap(),
                to_value(&color_to_int(&color)).unwrap(),
            ],
        );
    }
}

fn color_to_int(color: &impl Color32) -> u32 {
    let [r, g, b] = color.rgb();
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
