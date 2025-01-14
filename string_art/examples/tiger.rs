use string_art::{nails, Algorithm, ColorGroupSettings, ColorMapSettings, PercentageDarkness, Table};

fn main() {
    let mut art = Algorithm::new(
        Table::ellipse(
            image::open("C:\\projects\\rust\\string_art\\string_art\\examples\\tiger\\source.jpg")
                .unwrap()
                .resize(1000, 1000, image::imageops::FilterType::Lanczos3),
            nails::Circular::new(1.0),
            512,
        ),
        [
            ColorMapSettings::new(String::from("Black"), (0, 0, 0), 0, Default::default()),
            ColorMapSettings::new(String::from("Red"), (255, 0, 0), 0, Default::default()),
            ColorMapSettings::new(String::from("Yellow"), (255, 255, 0), 0, Default::default()),
            ColorMapSettings::new(
                String::from("White"),
                (255, 255, 255),
                0,
                Default::default(),
            ),
        ],
        20,
        PercentageDarkness(0.93),
        &[ColorGroupSettings::new(vec![0, 1, 2, 3], 0.5), ColorGroupSettings::new(vec![0], 0.5)],
        4000,
    )
    .unwrap();
    art.compute(4000).unwrap()
}
