use std::{fs::File, io::Write};

use palette::{rgb::Rgb, FromColor, IntoColor, Srgb};
use string_art::{geometry::circle, grid::Grid, nails::{self, circular::Direction}, Algorithm, ColorMapSettings, FlatDarkness, Lab, PercentageDarkness, Table};

fn main() {
    let mut art = Algorithm::new(
        Table::ellipse(
            image::open("string_art/examples/stag/source.jpg")
                .unwrap()
                .resize(1000, 1000, image::imageops::FilterType::Lanczos3)
                ,
            nails::Circular::new(0.3),
            512,
        ),
        [
            ColorMapSettings::new(String::from("Black"), (0, 0, 0), 0, Default::default())
        ],
        10,
        FlatDarkness(10.0),
        //RandomizedHookLogic::new(10),
    ).unwrap();
    for i in 1..41 {
        art.compute(i * 200).unwrap();
        let instructions = art.build_instructions();
        File::create(&format!("string_art/examples/stag/instructions_{}.txt", i)).and_then(|mut file| {
            file.write_all(&instructions.into_bytes())
        }).unwrap();
    }
}
