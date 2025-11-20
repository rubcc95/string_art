use crate::basic_types::*;
use string_art::math::Frac16;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct MonocolorPipeline(string_art::Monocolor<Frac16>);

#[wasm_bindgen]
impl MonocolorPipeline {
    #[wasm_bindgen(constructor)]
    pub fn new(image_buffer: &[u8]) -> Result<Self, WasmError> {
        Ok(Self(string_art::Monocolor::from_image(
            &image::load_from_memory(image_buffer).to_wasm()?,
        )))
    }

    pub fn build(self, settings: MonocolorSettings) -> Result<Computation, WasmError> {
        let board = string_art::ellipse::Ellipse::new(
            self.0.rect().as_(),
            string_art::nails::UniformCircular(settings.circular_nail_radius),
            settings.nail_count,
            settings.min_nail_distance,
        )
        .to_wasm()?;
        Ok(Computation::new(board, self.0, settings.decay))
    }
}

#[wasm_bindgen]
pub struct MonocolorSettings {
    pub decay: f32,
    min_nail_distance: usize,
    nail_count: usize,
    circular_nail_radius: f32,
}

#[wasm_bindgen]
impl MonocolorSettings {
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
