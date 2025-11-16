use geometry::Rect;
use string_art::computation::Pipeline;
use string_art::math::{Scalar32, ToFixed};
use string_art::monocolor::MonocolorIndex;
use string_art::nails::circular::Direction;
use string_art::nails::{Builder, UniformCircular};
use string_art::*;

fn main() {
    let image = image::open("string_art/examples/assets/tiger.jpg")
        .expect("Failed to open image")
        .resize(1024, 1024, image::imageops::FilterType::Lanczos3);

    let monocolor: Monocolor<nails::Anchor<nails::circular::Direction>, Scalar32> =
        Monocolor::from_image(&image, 3);

    let board = Ellipse::new(
        Rect::new(image.height() as f32, image.width() as f32),
        nails::UniformCircular(0.1),
        10,
        1,
    )
    .unwrap();

    let rect = Rect::new(image.height() as usize, image.width() as usize);

    let mut computation =
        Computation::new(monocolor, board, rect, decay::Flat(0.2.to_fixed())).unwrap();
    while let Some(step) = computation.next(){
        step.unwrap();
        // if let Ok(step) = step{
        //     println!("Step done from: {:?} to {:?}", step.anchor, computation.pipeline().color_map(&MonocolorIndex).anchor);
        //     println!("      Segment: {}", step.segment);
        // };
    }

    // let board = computation.board();
    // let n23 = board.nails[23];
    // let n4 = board.nails[4];
    // let c = nails::UniformCircular(5.0);
    // let res = c.create_segment((&n23, Direction::CLOCK_WISE), (&n4, Direction::CLOCK_WISE)).unwrap();
    // let res2 = c.create_segment( (&n4, Direction::CLOCK_WISE), (&n23, Direction::CLOCK_WISE)).unwrap();

    // println!("1 -> {res}");
    // println!("2 -> {res2}");
}