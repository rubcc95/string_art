mod float;
pub use float::Float;

pub mod geometry {
    pub mod circle;
    pub mod point;
    pub mod segment;

    pub use circle::Circle;
    pub use point::Point;
    pub use segment::Segment;
}

pub mod grid;
pub mod hooks;
pub use grid::Grid;

// fn get_pixels_between(p1: Point<f64>, p2: Point<f64>) -> impl Iterator<Item = Point<isize>> {
//     Bresenham::new(
//         (p1.x as isize, p1.y as isize),
//         (p2.x as isize, p2.y as isize),
//     )
//     .map(|(x, y)| Point { x, y })
// }

pub mod multi_color;
pub mod ditherer {
    use std::ops::Deref;

    use crate::{geometry::Point, Float, Grid};
    use image::{GenericImageView, Pixel, Rgb, RgbImage};
    use num_traits::AsPrimitive;
    use palette::{color_difference::EuclideanDistance, FromColor, Srgb};
    type Lab<T> = palette::Lab<palette::white_point::D65, T>;

    #[derive(Clone, Copy)]
    pub struct PaletteColor<T> {
        pub index: usize,
        color: Lab<T>,
    }

    impl<T: Copy> PaletteColor<T> {
        pub unsafe fn from_palette_unchecked(palette: Vec<Lab<T>>, index: usize) -> Self {
            Self {
                index,
                color: *palette.get_unchecked(index),
            }
        }

        pub fn from_palette(palette: Vec<Lab<T>>, index: usize) -> Self {
            Self {
                index,
                color: palette[index],
            }
        }

        #[allow(invalid_value)]
        pub fn from_color(color: Lab<T>) -> Self {
            Self {
                index: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
                color,
            }
        }
    }
    #[derive()]
    pub struct DitherData<T> {
        pixels: Vec<PaletteColor<T>>,
        grid: Grid,
    }

    impl<T> DitherData<T> {
        pub fn pixels(&self) -> &[PaletteColor<T>] {
            &self.pixels
        }

        fn get(&self, point: Point<usize>) -> Option<&PaletteColor<T>> {
            self.index_of(point)
                .map(|index| unsafe { self.pixels.get_unchecked(index) })
        }

        fn get_mut(&mut self, point: Point<usize>) -> Option<&mut PaletteColor<T>> {
            self.index_of(point)
                .map(|index| unsafe { self.pixels.get_unchecked_mut(index) })
        }

        unsafe fn get_unchecked(&self, point: Point<usize>) -> &PaletteColor<T> {
            self.pixels.get_unchecked(self.index_of_unchecked(point))
        }

        unsafe fn get_unchecked_mut(&mut self, point: Point<usize>) -> &mut PaletteColor<T> {
            self.pixels
                .get_unchecked_mut(self.grid.index_of_unchecked(point))
        }
    }

    impl<T> Deref for DitherData<T> {
        type Target = Grid;

        fn deref(&self) -> &Self::Target {
            &self.grid
        }
    }

    impl<T: Float> DitherData<T> {
        pub fn from_image(image: &impl GenericImageView<Pixel: Pixel<Subpixel = T>>) -> Self {
            Self {
                pixels: image
                    .pixels()
                    .map(|(_, _, color)| {
                        let pixels = color.to_rgb();
                        let srgb = Srgb::new(pixels[0], pixels[1], pixels[2]);
                        let lab = Lab::from_color(srgb);
                        PaletteColor::from_color(lab)
                    })
                    .collect(),
                grid: Grid::new(image.height() as usize, image.width() as usize),
            }
        }
    }

    pub struct DitherWeight<T> {
        pub pos: Point<isize>,
        pub weight: T,
    }

    impl<T: Float> DitherWeight<T> {
        fn get_mut<'a>(
            &self,
            pos: Point<usize>,
            image_dithered: &'a mut DitherData<T>,
        ) -> Option<&'a mut PaletteColor<T>> {
            (pos.as_::<isize>() + self.pos)
                .cast::<usize>()
                .and_then(|point| {
                    let res = image_dithered.get_mut(point);
                    res
                })
        }

        fn apply(&self, pos: Point<usize>, color: Lab<T>, image_dithered: &mut DitherData<T>) {
            if let Some(pixel) = self.get_mut(pos, image_dithered) {
                pixel.color.l += color.l * self.weight;
                pixel.color.a += color.a * self.weight;
                pixel.color.b += color.b * self.weight;
            }
        }
    }

    #[derive(Debug)]
    pub struct Ditherer<T, W> {
        palette: Vec<Lab<T>>,
        weights: W,
    }

    impl<T, W> Ditherer<T, W> {
        pub fn new(palette: Vec<Lab<T>>, weights: W) -> Self {
            Ditherer { palette, weights }
        }
    }
    impl<T: Float, W: AsRef<[DitherWeight<T>]>> Ditherer<T, W> {
        pub fn dither(&self, image_dithered: &mut DitherData<T>) {
            let y = image_dithered.height;
            let x = image_dithered.width;

            for y in 0..y {
                for x in 0..x {
                    let old_color = unsafe { image_dithered.get_unchecked_mut(Point { x, y }) };

                    let color = self.find_closest_color(&old_color.color);
                    let color_diff = Lab::new(
                        old_color.color.l - color.color.l,
                        old_color.color.a - color.color.a,
                        old_color.color.b - color.color.b,
                    );

                    *old_color = color;
                    for dither_weight in self.weights.as_ref() {
                        dither_weight.apply(Point { x, y }, color_diff, image_dithered);
                    }
                }
            }
        }

        fn find_closest_color(&self, color: &Lab<T>) -> PaletteColor<T> {
            let mut min = T::INFINITY;
            let mut best = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
            for (index, &palette_color) in self.palette.iter().enumerate() {
                let distance = color.distance_squared(palette_color);
                if distance < min {
                    min = distance;
                    best = PaletteColor {
                        index,
                        color: palette_color,
                    };
                }
            }
            best
            // self.palette
            //     .iter()
            //     .min_by(|p1, p2| {
            //         let diff1: T = (0..3).map(|i| (color[i] - p1[i]).powi(2)).sum();
            //         let diff2: T = (0..3).map(|i| (color[i] - p2[i]).powi(2)).sum();
            //         diff1.partial_cmp(&diff2).unwrap()
            //     })
            //     .unwrap()
            //     .clone()
        }
    }

    impl<T: Float> Ditherer<T, [DitherWeight<T>; 4]> {
        pub fn floyd_steinberg(palette: Vec<Lab<T>>) -> Self {
            Self::new(palette, T::FLOYD_STEINBERG)
        }
    }

    fn convert_images_to_dither_input<T: Float>(image: &RgbImage) -> DitherData<T>
    where
        u8: AsPrimitive<T>,
    {
        let height = image.height() as usize;
        let width = image.width() as usize;

        let mut result = vec![vec![Lab::new(T::ZERO, T::ZERO, T::ZERO); width]; height];
        DitherData {
            pixels: image
                .pixels()
                .map(|pixel| {
                    let srgb = Srgb::new(
                        pixel[0].as_() / T::TWO_FIVE_FIVE,
                        pixel[1].as_() / T::TWO_FIVE_FIVE,
                        pixel[2].as_() / T::TWO_FIVE_FIVE,
                    );
                    let lab = Lab::from_color(srgb);
                    PaletteColor::from_color(lab)
                })
                .collect(),
            grid: Grid::new(height, width),
        }
    }

    fn convert_dither_output_to_images<T: Float>(
        dither_output: DitherData<T>,
        width: u32,
        height: u32,
    ) -> RgbImage {
        let grid = dither_output.grid;
        let buffer: Vec<_> = dither_output
            .pixels
            .into_iter()
            .map(|pixel| {
                let srgb = Srgb::from_color(pixel.color);
                [
                    (srgb.red * T::TWO_FIVE_FIVE).to_u8().unwrap(),
                    (srgb.green * T::TWO_FIVE_FIVE).to_u8().unwrap(),
                    (srgb.blue * T::TWO_FIVE_FIVE).to_u8().unwrap(),
                ]
                .into_iter()
            })
            .flatten()
            .collect();
        RgbImage::from_raw(grid.width as u32, grid.height as u32, buffer).unwrap()
    }
    fn count_colors_kmeans<T: Float>(
        image: impl Iterator<Item = Lab<T>>,
        palette: &[Lab<T>],
    ) -> Vec<usize> {
        let mut counts = vec![0; palette.len()];
        for pixel in image {
            let mut min_dist = T::INFINITY;
            let mut closest_color = 0;
            for (i, &color) in palette.iter().enumerate() {
                let dist = pixel.distance_squared(color); // Define esta función
                if dist < min_dist {
                    min_dist = dist;
                    closest_color = i;
                }
            }
            counts[closest_color] += 1;
        }
        counts
    }
    #[test]
    fn test() -> Result<(), image::ImageError> {
        // Cargar algunas imágenes de ejemplo
        let image = image::open("examples/alba.jpg")?.to_rgb32f();

        // Crear una instancia de FSDither
        let palette = vec![
            Lab::new(0.0, 0.0, 0.0),                // Negro en Lab
            Lab::new(53.23288, 80.10933, 67.22006), // Rojo en Lab
            Lab::new(100.0, 0.0, -0.0),             // Blanco en Lab
        ];
        let dither = Ditherer::floyd_steinberg(palette.clone());

        // Convertir las imágenes al formato de entrada del dither
        let mut dither_data = DitherData::from_image(&image);
        let mut dither_data_2 = dither_data.pixels.clone();

        // Aplicar el dithering
        dither.dither(&mut dither_data);

        // Convertir el resultado de vuelta a imágenes
        //let image = convert_dither_output_to_images(dither_data, image.width(), image.height());
        //image.save("output.png").unwrap();
        let mut count1: Vec<_> = dither.palette.iter().map(|_| 0).collect();
        let count2 =
            count_colors_kmeans(dither_data_2.into_iter().map(|color| color.color), &palette);
        for &pixel in dither_data.pixels().iter() {
            *unsafe { count1.get_unchecked_mut(pixel.index) } += 1;
        }
        // for &pixel in dither_data_2.iter() {
        //     let mut min = f32::INFINITY;
        //     let mut best = 10;

        //     for (idx, (_, color)) in count2.iter().enumerate() {
        //         let dt = color.distance_squared(pixel.color);
        //         if dt < min {
        //             min = dt;
        //             best = idx;
        //         }
        //     }
        //     unsafe { count2.get_unchecked_mut(best).0 += 1 };
        // }

        for (idx, pixel) in count1.iter().enumerate() {
            println!("{}: {}", idx, pixel);
        }
        println!();
        for (idx, pixel) in count2.iter().enumerate() {
            println!("{}: {}", idx, pixel);
        }
        Ok(())
    }
}
