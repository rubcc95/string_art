use derive_more::*;
use string_art_grid::Grid;


#[derive(Clone, Debug, Deref, DerefMut, From)]
pub struct Image<C = (f32, f32, f32)>(pub Grid<C>);

#[cfg(feature = "image")]
mod from_image {
    use super::*;    
    use string_art_geometry::Rect;
    use image::*;

    impl From<DynamicImage> for Image {
        fn from(value: DynamicImage) -> Self {
            Self(unsafe {
                Grid::from_raw(
                    value
                        .pixels()
                        .map(|(_, _, pixel)| {
                            let [r, g, b, _] = pixel.0;
                            (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
                        })
                        .collect(),
                    Rect::new(value.height() as usize, value.width() as usize),
                )
            })
        }
    }

    impl From<RgbImage> for Image {
        fn from(value: RgbImage) -> Self {
            Self(unsafe {
                Grid::from_raw(
                    value
                        .pixels()
                        .map(|pixel| {
                            let [r, g, b] = pixel.0;
                            (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
                        })
                        .collect(),
                    Rect::new(value.height() as usize, value.width() as usize),
                )
            })
        }
    }

    impl From<RgbaImage> for Image {
        fn from(value: RgbaImage) -> Self {
            Self(unsafe {
                Grid::from_raw(
                    value
                        .pixels()
                        .map(|pixel| {
                            let [r, g, b, _] = pixel.0;
                            (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
                        })
                        .collect(),
                    Rect::new(value.height() as usize, value.width() as usize),
                )
            })
        }
    }

    impl From<Rgb32FImage> for Image {
        fn from(value: Rgb32FImage) -> Self {
            Self(unsafe {
                Grid::from_raw(
                    value.pixels().map(|pixel| pixel.0.into()).collect(),
                    Rect::new(value.height() as usize, value.width() as usize),
                )
            })
        }
    }

    impl From<Rgba32FImage> for Image {
        fn from(value: Rgba32FImage) -> Self {
            Self(unsafe {
                Grid::from_raw(
                    value
                        .pixels()
                        .map(|pixel| {
                            let [r, g, b, _] = pixel.0;
                            (r, g, b)
                        })
                        .collect(),
                    Rect::new(value.height() as usize, value.width() as usize),
                )
            })
        }
    }
}
