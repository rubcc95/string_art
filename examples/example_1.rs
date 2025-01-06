use image::Rgb;
use string_art::{
    hooks,
    multi_color::StringArt,
};

fn main() {
    let mut art = StringArt::ellipse(
        &image::open("examples/output.png").unwrap().resize(
            2000,
            2000,
            image::imageops::FilterType::Lanczos3,
        ),
        hooks::Circular::new(2.1),
        257,
        [
            Rgb([0, 0, 0]),       //3417
            // Rgb([255, 255, 255]), //88
            // Rgb([255, 0, 0]),     // Rgb([142, 113, 73]),    //3254
                                  // Rgb([162, 37, 71]),     //1364
                                  //Rgb([201, 186, 163])    //1877
        ]
        .into_iter(),
        10,
        0.9,
    );
    art.show_weigths();
    for i in 1..21 {
        art.compute_steps(i * 300);
        art.save_image(&format!("examples/example_1/output_{}.png", i), 1.5)
            .unwrap();
    }
}
