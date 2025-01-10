use palette::{rgb::Rgb, FromColor, IntoColor, Srgb};
use string_art::{nails, Lab, PercentageDarkness,StringArt, Table};

fn main() {
    // let mut art = StringArt::new(
    //     Table::ellipse(
    //         image::open("examples/tiger/source.jpg")
    //             .unwrap()
    //             .resize(1000, 1000, image::imageops::FilterType::Lanczos3)
    //             .into(),
    //         hooks::Circular::new(1.5),
    //         512,
    //     ),
    //     [
    //         Lab::from_color(Srgb::new(1.0, 1.0, 1.0)),
    //         Lab::from_color(Srgb::new(0.0, 0.0, 0.0)),
    //         Lab::from_color(Srgb::new(1.0, 0.0, 0.0)),            
    //         Lab::from_color(Srgb::new(1.0, 0.5098, 0.0)),
    //     ],
    //     80,
    //     PercentageDarkness(0.97),
    // );
    // for i in 1..41 {
    //     art.compute(i * 200);
    //     art.save_image_svg(&format!("examples/tiger/output_{}.svg", i), 0.5)
    //         .unwrap();
    // }
}
