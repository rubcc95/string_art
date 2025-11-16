use derive_more::*;
use image::RgbImage;
use std::fmt;

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Input {
    pub board_shape: BoardShape,
    pub nail_shape: NailShape,
    pub computation_settings: ComputationSettings,
    pub pipeline: pipeline::Mode,
    pub image: Image,
}

pub type InstructionsBuilder<W> = fn(writer: &mut W, anchor: &super::Anchor) -> fmt::Result;

impl Input {
    pub fn instrucions_builder<W: fmt::Write>(&self) -> InstructionsBuilder<W> {
        fn builder_fn<W: fmt::Write, L: super::CompatibleLink>(
            writer: &mut W,
            anchor: &super::Anchor,
        ) -> fmt::Result {
            write!(writer, "{} {}\n", anchor.index, L::from_idx(anchor.link))
        }

        match &self.nail_shape {
            NailShape::Circular(_) => builder_fn::<_, string_art::nails::circular::Direction>,
            NailShape::Point => builder_fn::<_, string_art::nails::point::Link>,
            NailShape::Hook(_) => |_, _| unimplemented!(),
        }
    }
}

#[derive(Clone, PartialEq, Deref, DerefMut, From)]
pub struct Image(RgbImage);

#[cfg(feature = "serde")]
impl serde::Serialize for Image {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let width = self.0.width();
        let height = self.0.height();
        let pixels = self.0.as_raw(); // &[u8]

        let mut state = serializer.serialize_struct("SerializedImage", 3)?;
        state.serialize_field("width", &width)?;
        state.serialize_field("height", &height)?;
        state.serialize_field("pixels", pixels)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct ImageData {
            width: u32,
            height: u32,
            pixels: Vec<u8>,
        }

        let helper = ImageData::deserialize(deserializer)?;

        match image::ImageBuffer::from_vec(helper.width, helper.height, helper.pixels) {
            Some(img) => Ok(Image(img)),
            None => Err(serde::de::Error::custom("Invalid image buffer")),
        }
    }
}

pub use board_shape::BoardShape;

pub mod board_shape {
    use super::*;

    #[derive(Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum BoardShape {
        Ellipse(Ellipse),
        Rectangle(Rectangle),
    }

    impl Default for BoardShape {
        fn default() -> Self {
            Self::Ellipse(Default::default())
        }
    }

    pub use rectangle::Rectangle;

    pub mod rectangle {
        use super::*;

        #[derive(Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum Rectangle {
            Auto(Auto),
            Manual(Manual),
        }

        #[derive(Clone, Copy, PartialEq, Deref, DerefMut)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct Auto(pub usize);

        impl Default for Auto {
            fn default() -> Self {
                Self(512)
            }
        }

        #[derive(Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct Manual {
            pub width: usize,
            pub height: usize,
        }

        impl Default for Manual {
            fn default() -> Self {
                Self {
                    width: 256,
                    height: 256,
                }
            }
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct Ellipse {
        pub nail_count: usize,
        pub min_nail_distance: usize,
    }

    impl Default for Ellipse {
        fn default() -> Self {
            Self {
                nail_count: 512,
                min_nail_distance: 20,
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NailShape {
    Circular(f32),
    Hook(f32),
    Point,
}

impl Default for NailShape {
    fn default() -> Self {
        Self::Circular(0.234375)
    }
}

pub mod pipeline {

    #[derive(Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum Mode {
        Mono(u32),
        Multi,
    }

    impl Default for Mode {
        fn default() -> Self {
            Self::Mono(5000)
        }
    }
}

pub use computation_settings::ComputationSettings;
pub mod computation_settings {

    #[derive(PartialEq, Clone, Copy)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct ComputationSettings {
        pub decay_fn: DecayFn,
        pub resolution: u32,
        pub precision: Precision,
    }

    impl Default for ComputationSettings {
        fn default() -> Self {
            Self {
                decay_fn: Default::default(),
                resolution: 1024,
                precision: Default::default(),
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum Precision {
        P8,
        P16,
        #[default]
        P32,
        P64,
    }

    #[derive(Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub enum DecayFn {
        Flat(f32),
        Percentage(f32),
    }

    impl Default for DecayFn {
        fn default() -> Self {
            Self::Flat(0.3125)
        }
    }
}
