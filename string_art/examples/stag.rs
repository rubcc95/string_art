use palette::{rgb::Rgb, FromColor, IntoColor, Srgb};
use string_art::{nails, FlatDarkness, Lab, PercentageDarkness, StringArt, Table};

fn main() {
    // let mut art = StringArt::new(
    //     Table::ellipse(
    //         image::open("examples/stag/source.jpg")
    //             .unwrap()
    //             .resize(1000, 1000, image::imageops::FilterType::Lanczos3)
    //             .into(),
    //         hooks::Circular::new(0.3),
    //         700,
    //     ),
    //     [
    //         ("White", Lab::from_color(Srgb::new(1.0, 1.0, 1.0)), 700/4),
    //         ("Light blue", Lab::from_color(Srgb::new(0.0, 0.8431, 0.8824)), 0),
    //         ("Blue", Lab::from_color(Srgb::new(0.0, 0.4706, 0.9412)), 0),
    //         ("Dark blue", Lab::from_color(Srgb::new(0.0, 0.0, 0.4706)), 0),
    //         ("Black", Lab::from_color(Srgb::new(0.0, 0.0, 0.0)), 0),
    //     ].iter(),
    //     10,
    //     FlatDarkness(10.0),
    //     //RandomizedHookLogic::new(10),
    // );
    // for i in 1..41 {
    //     art.compute(i * 200);
    //     art.save_image_svg(&format!("examples/stag/output_{}.svg", i), 1.0)
    //         .unwrap();
    // }
}
