use num_traits::AsPrimitive;
use palette::FromColor;

use crate::Float;

pub type Lab<S> = palette::Lab<palette::white_point::D65, S>;

pub type Rgb = (u8, u8, u8);

pub trait AsLab<S>{
    fn as_lab(&self) -> Lab<S>;
}

impl<S: Float> AsLab<S> for Rgb where u8: AsPrimitive<S>{
    fn as_lab(&self) -> Lab<S>{
        Lab::from_color(palette::Srgb::new(
            self.0.as_() / S::TWO_FIVE_FIVE,
            self.1.as_() / S::TWO_FIVE_FIVE,
            self.2.as_() / S::TWO_FIVE_FIVE,
        ))
    }
}
