use std::ops::{Add, Div, Mul, Sub};

use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};

use crate::Float;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb<T = u8>(pub T, pub T, pub T);

impl<T: Copy + Mul<Output = T> + Add<Output = T> + Sub<Output = T>> Rgb<T> {
    pub fn distance_squared(&self, other: &Self) -> T {
        let r = self.0 - other.0;
        let g = self.1 - other.1;
        let b = self.2 - other.2;
        r * r + g * g + b * b
    }
}

impl<T: Float> Rgb<T>{
    pub fn distance(&self, other: &Self) -> T{
        num_traits::Float::sqrt(self.distance_squared(other))
    }
}

impl<T: Add<Output = T>> Add for Rgb<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Rgb(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T: Copy> From<Rgb<T>> for [T; 3]{
    fn from(rgb: Rgb<T>) -> Self{
        [rgb.0, rgb.1, rgb.2]
    }
}

impl<T: Copy> From<[T; 3]> for Rgb<T>{
    fn from(value: [T; 3]) -> Self {
        Rgb(value[0], value[1], value[2])
    }
}

impl<T: Sub<Output = T>> Sub for Rgb<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Rgb(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T: Mul<Output = T>> Mul for Rgb<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Rgb(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
    }
}

impl<T: Div<Output = T>> Div for Rgb<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Rgb(self.0 / rhs.0, self.1 / rhs.1, self.2 / rhs.2)
    }
}

impl<T: Copy> AsRgb<T> for Rgb<T> {
    fn as_rgb(&self) -> Rgb<T> {
        *self
    }
}

impl<S: Float> AsRgb<S> for Rgb where u8: AsPrimitive<S>{
    fn as_rgb(&self) -> Rgb<S> {
        Rgb(
            self.0.as_() / S::TWO_FIVE_FIVE,
            self.1.as_() / S::TWO_FIVE_FIVE,
            self.2.as_() / S::TWO_FIVE_FIVE,
        )
    }
}

pub trait AsRgb<T> {
    fn as_rgb(&self) -> Rgb<T>;
}

// pub trait AsLab<S>{
//     fn as_lab(&self) -> Lab<S>;
// }

// impl<S: Float> AsLab<S> for Rgb where u8: AsPrimitive<S>{
//     fn as_lab(&self) -> Lab<S>{
//         Lab::from_color(palette::Srgb::new(
//             self.0.as_() / S::TWO_FIVE_FIVE,
//             self.1.as_() / S::TWO_FIVE_FIVE,
//             self.2.as_() / S::TWO_FIVE_FIVE,
//         ))
//     }
// }
