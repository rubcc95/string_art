use geometry::Rect;
use string_art::*;
use string_art_math::Frac16;

fn main() {
    let image = image::open("string_art/examples/assets/tiger.jpg")
        .expect("Failed to open image")
        .resize(1024, 1024, image::imageops::FilterType::Lanczos3);

    let monocolor = Monocolor::from_image(&image);

    let board = Ellipse::new(
        Rect::new(image.width() as f32, image.height() as f32),
        nails::UniformCircular(0.1),
        10,
        1,
    )
    .unwrap();

    let rect = Rect::new(image.height() as usize, image.width() as usize);
    for step in Computation::new(monocolor, board, Frac16::from_bits(5000)) {}
}
