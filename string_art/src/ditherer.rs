use palette::color_difference::EuclideanDistance;

use crate::{colors::LabImage, geometry::Point, Float, Lab};

pub struct DitherWeight<T> {
    pub pos: Point<isize>,
    pub weight: T,
}

impl<T: Float> DitherWeight<T> {
    fn get_mut<'a>(
        &self,
        pos: Point<usize>,
        image_dithered: &'a mut LabImage<T>,
    ) -> Option<&'a mut Lab<T>> {
        (pos.as_::<isize>() + self.pos)
            .cast::<usize>()
            .and_then(|point| {
                let res = image_dithered.get_mut(point);
                res
            })
    }

    fn apply(&self, pos: Point<usize>, color: Lab<T>, image_dithered: &mut LabImage<T>) {
        if let Some(pixel) = self.get_mut(pos, image_dithered) {
            pixel.l += color.l * self.weight;
            pixel.a += color.a * self.weight;
            pixel.b += color.b * self.weight;
        }
    }
}

pub trait WeightedColor<T> {
    fn color(&self) -> Lab<T>;

    fn add_weight(&mut self);
}

pub struct Ditherer<'a, P, W> {
    palette: &'a mut [P],
    weights: W,
}

impl<'a, P, W> Ditherer<'a, P, W> {
    pub fn new(palette: &'a mut [P], weigths: W) -> Self
    {
        Self {
            palette,
            weights: weigths,
        }
    }
}
impl<'a, P, T: Float> Ditherer<'a, P, [DitherWeight<T>; 4]> {
    pub fn floyd_steinberg(palette: &'a mut [P]) -> Self {
        Self::new(palette, T::FLOYD_STEINBERG)
    }
}

impl<'a, P, W> Ditherer<'a, P, W> {
    pub fn dither<T: Float>(&mut self, image_dithered: &mut LabImage<T>)
    where
        P: WeightedColor<T>,
        W: AsRef<[DitherWeight<T>]>,
    {
        let y = image_dithered.height;
        let x = image_dithered.width;

        for y in 0..y {
            for x in 0..x {
                let old_color = unsafe { image_dithered.get_unchecked_mut(Point { x, y }) };

                let color = self.find_closest_color(old_color);

                let color_diff = Lab::new(
                    old_color.l - color.l,
                    old_color.a - color.a,
                    old_color.b - color.b,
                );

                *old_color = color;
                for dither_weight in self.weights.as_ref() {
                    dither_weight.apply(Point { x, y }, color_diff, image_dithered);
                }
            }
        }
    }

    fn find_closest_color<T: Float>(&mut self, color: &Lab<T>) -> Lab<T>
    where
        P: WeightedColor<T>,
    {
        let mut iter = self.palette.iter_mut();
        if let Some(weighted_color) = iter.next() {
            let mut min = color.distance_squared(weighted_color.color());
            let mut best = weighted_color;
            while let Some(weighted_color) = iter.next() {
                let distance = color.distance_squared(weighted_color.color());
                if distance < min {
                    min = distance;
                    best = weighted_color
                }
            }

            best.add_weight();
            best.color()
        } else{
            panic!("Palette is empty!")
        }
    }
}

// impl<T, W> Ditherer<T, W> {
//     pub fn new(palette: Vec<Lab<T>>, weights: W) -> Self {
//         Ditherer { palette, weights }
//     }
// }
// impl<T: Float, W: AsRef<[DitherWeight<T>]>> Ditherer<T, W> {
//     pub fn dither(&self, image_dithered: &mut DitherData<T>) {
//         let y = image_dithered.height;
//         let x = image_dithered.width;

//         for y in 0..y {
//             for x in 0..x {
//                 let old_color = unsafe { image_dithered.get_unchecked_mut(Point { x, y }) };

//                 let color = self.find_closest_color(&old_color.color);
//                 let color_diff = Lab::new(
//                     old_color.color.l - color.color.l,
//                     old_color.color.a - color.color.a,
//                     old_color.color.b - color.color.b,
//                 );

//                 *old_color = color;
//                 for dither_weight in self.weights.as_ref() {
//                     dither_weight.apply(Point { x, y }, color_diff, image_dithered);
//                 }
//             }
//         }
//     }

//     fn find_closest_color(&self, color: &Lab<T>) -> PaletteColor<T> {
//         let mut min = T::INFINITY;
//         let mut best = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
//         for (index, &palette_color) in self.palette.iter().enumerate() {
//             let distance = color.distance_squared(palette_color);
//             if distance < min {
//                 min = distance;
//                 best = PaletteColor {
//                     index,
//                     color: palette_color,
//                 };
//             }
//         }
//         best
//     }
// }

// fn convert_images_to_dither_input<T: Float>(image: &RgbImage) -> DitherData<T>
// where
//     u8: AsPrimitive<T>,
// {
//     let height = image.height() as usize;
//     let width = image.width() as usize;

//     let mut result = vec![vec![Lab::new(T::ZERO, T::ZERO, T::ZERO); width]; height];
//     DitherData {
//         pixels: image
//             .pixels()
//             .map(|pixel| {
//                 let srgb = Srgb::new(
//                     pixel[0].as_() / T::TWO_FIVE_FIVE,
//                     pixel[1].as_() / T::TWO_FIVE_FIVE,
//                     pixel[2].as_() / T::TWO_FIVE_FIVE,
//                 );
//                 let lab = Lab::from_color(srgb);
//                 PaletteColor::from_color(lab)
//             })
//             .collect(),
//         grid: Grid::new(height, width),
//     }
// }

// fn convert_dither_output_to_images<T: Float>(
//     dither_output: DitherData<T>,
//     width: u32,
//     height: u32,
// ) -> RgbImage {
//     let grid = dither_output.grid;
//     let buffer: Vec<_> = dither_output
//         .pixels
//         .into_iter()
//         .map(|pixel| {
//             let srgb = Srgb::from_color(pixel.color);
//             [
//                 (srgb.red * T::TWO_FIVE_FIVE).to_u8().unwrap(),
//                 (srgb.green * T::TWO_FIVE_FIVE).to_u8().unwrap(),
//                 (srgb.blue * T::TWO_FIVE_FIVE).to_u8().unwrap(),
//             ]
//             .into_iter()
//         })
//         .flatten()
//         .collect();
//     RgbImage::from_raw(grid.width as u32, grid.height as u32, buffer).unwrap()
// }
// fn count_colors_kmeans<T: Float>(
//     image: impl Iterator<Item = Lab<T>>,
//     palette: &[Lab<T>],
// ) -> Vec<usize> {
//     let mut counts = vec![0; palette.len()];
//     for pixel in image {
//         let mut min_dist = T::INFINITY;
//         let mut closest_color = 0;
//         for (i, &color) in palette.iter().enumerate() {
//             let dist = pixel.distance_squared(color); // Define esta función
//             if dist < min_dist {
//                 min_dist = dist;
//                 closest_color = i;
//             }
//         }
//         counts[closest_color] += 1;
//     }
//     counts
// }
// #[test]
// fn test() -> Result<(), image::ImageError> {
//     // Cargar algunas imágenes de ejemplo
//     let image = image::open("examples/alba.jpg")?.to_rgb32f();

//     // Crear una instancia de FSDither
//     let palette = vec![
//         Lab::new(0.0, 0.0, 0.0),                // Negro en Lab
//         Lab::new(53.23288, 80.10933, 67.22006), // Rojo en Lab
//         Lab::new(100.0, 0.0, -0.0),             // Blanco en Lab
//     ];
//     let dither = Ditherer::floyd_steinberg(&mut palette);

//     // Convertir las imágenes al formato de entrada del dither
//     let mut dither_data = DitherData::from_image(&image);
//     let mut dither_data_2 = dither_data.pixels.clone();

//     // Aplicar el dithering
//     dither.dither(&mut dither_data);

//     // Convertir el resultado de vuelta a imágenes
//     //let image = convert_dither_output_to_images(dither_data, image.width(), image.height());
//     //image.save("output.png").unwrap();
//     let mut count1: Vec<_> = dither.palette.iter().map(|_| 0).collect();
//     let count2 =
//         count_colors_kmeans(dither_data_2.into_iter().map(|color| color.color), &palette);
//     for &pixel in dither_data.pixels().iter() {
//         *unsafe { count1.get_unchecked_mut(pixel.index) } += 1;
//     }
//     // for &pixel in dither_data_2.iter() {
//     //     let mut min = f32::INFINITY;
//     //     let mut best = 10;

//     //     for (idx, (_, color)) in count2.iter().enumerate() {
//     //         let dt = color.distance_squared(pixel.color);
//     //         if dt < min {
//     //             min = dt;
//     //             best = idx;
//     //         }
//     //     }
//     //     unsafe { count2.get_unchecked_mut(best).0 += 1 };
//     // }

//     for (idx, pixel) in count1.iter().enumerate() {
//         println!("{}: {}", idx, pixel);
//     }
//     println!();
//     for (idx, pixel) in count2.iter().enumerate() {
//         println!("{}: {}", idx, pixel);
//     }
//     Ok(())
// }
