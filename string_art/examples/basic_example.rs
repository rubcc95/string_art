use geometry::Rect;
use string_art::*;
use string_art_math::Frac16;

fn main() {
    let image = image::open("string_art/examples/kitty.jpg")
        .expect("Failed to open image")
        .resize(1024, 1024, image::imageops::FilterType::Lanczos3);

    let monocolor = Monocolor::from_image(&image::load_from_memory(&[]).unwrap());

    let ellipse = Ellipse::new(
        Rect::new(image.width() as f32, image.height() as f32),
        nails::UniformCircular(0.2),
        512,
    );

    let board = ellipse::Board::new(&ellipse, 16).unwrap();

    for step in Computation::new(monocolor, board, Frac16::from_bits(5000)) {
        println!(
            "Step: segment=(({:.2}, {:.2}) -> ({:.2}, {:.2})), nail={}, link={}",
            step.segment.start.x,
            step.segment.start.y,
            step.segment.end.x,
            step.segment.end.y,
            step.anchor.idx,
            step.anchor.link
        )
    }
}
