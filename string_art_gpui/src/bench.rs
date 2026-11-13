use string_art::geometry::Rect;
use string_art::nails::{self, circular::Direction};
use string_art::sync::Parallel;
use string_art::*;

const SIZE: u32 = 1024;
const NAILS: usize = 512;
const MIN_DISTANCE: usize = 20;
const STEPS: u32 = 3000;

fn test_image(size: u32) -> image::RgbImage {
    let center = size as f32 / 2.0;
    image::RgbImage::from_fn(size, size, |x, y| {
        let dx = x as f32 - center;
        let dy = y as f32 - center;
        let v = (dx.hypot(dy) / center * 255.0).min(255.0) as u8;
        image::Rgb([v, v, v])
    })
}

pub fn run<X: Parallel>(par: X) {
    let image = test_image(SIZE);
    let monocolor: Monocolor<nails::Anchor<Direction>> = Monocolor::from_image(&image, STEPS);
    let board = Ellipse::new(
        Rect::new(SIZE as f32, SIZE as f32),
        nails::UniformCircular(0.1),
        NAILS,
        MIN_DISTANCE,
    )
    .expect("tablero inválido");
    let rect = Rect::new(SIZE as usize, SIZE as usize);

    let computation = Computation::new(
        par,
        monocolor,
        board,
        rect,
        decay::Decay::Flat(Frac::from_num(0.2)),
    )
    .expect("el mapa de color no cuadra con el área");

    computation
        .map(|step| step.expect("paso fallido"))
        .for_each(drop);
}
