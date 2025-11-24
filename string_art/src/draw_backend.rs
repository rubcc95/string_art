use string_art_color::Color32;
use string_art_geometry::{Circle, Segment};

pub trait DrawBackend {
    fn draw_circle(&mut self, circle: Circle<f32>, color: impl Color32);

    fn draw_circunference(&mut self, circle: Circle<f32>, stroke: f32, color: impl Color32);

    fn draw_segment(&mut self, segment: Segment<f32>, stroke: f32, color: impl Color32);
}
