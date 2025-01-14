use std::{fmt::Display, ops::{AddAssign, DivAssign, MulAssign, SubAssign}};

use num_traits::{ConstOne, ConstZero};

use crate::{ditherer::DitherWeight, geometry::Point, Lab};

pub trait Float:
    'static
    + Display
    + Sync
    + Send
    + Into<svg::node::Value>
    + image::Primitive
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + num_traits::Float
    + num_traits::NumCast
    + ConstZero
    + ConstOne
    + palette::num::MulSub
    + palette::num::PartialCmp
    + palette::num::Clamp
    + palette::num::Real
    + palette::num::Zero
    + palette::num::One
    + palette::num::FromScalar<Scalar = Self>
    + palette::num::Recip
    + palette::num::IsValidDivisor
    + palette::bool_mask::HasBoolMask<Mask = bool>
    + palette::num::Arithmetics
    + palette::num::Powf
    + palette::num::Powi
    + palette::num::Cbrt
    + palette::num::MulAdd
    + palette::num::Sqrt
{
    const HALF: Self;
    const EPSILON: Self;
    const INFINITY: Self;
    const TWO: Self;
    const PI: Self;
    const SQRT140050: Self; // max euclidean distance for a LAB color
    const HALF_SQRT140050: Self;
    const TWO_FIVE_FIVE: Self;
    const FLOYD_STEINBERG: [DitherWeight<Self>; 4];
    const MINUS_HUNDRED_TWENTY_EIGHT: Self;
    const HUNDRED_TWENTY_SEVEN: Self;
    const HUNDRED: Self;

    const RED: Lab<Self>;
    const GREEN: Lab<Self>;
    const BLUE: Lab<Self>;
    const YELLOW: Lab<Self>;
    const BLACK: Lab<Self>;
    const WHITE: Lab<Self>;
    const GRAY: Lab<Self>;
    const GREY: Lab<Self> = Self::GRAY;
    const ORANGE: Lab<Self>;
    const PURPLE: Lab<Self>;
    const BROWN: Lab<Self>;
    const PINK: Lab<Self>;
    const CYAN: Lab<Self>;
    const MAGENTA: Lab<Self>;
    const LIME: Lab<Self>;
    const TEAL: Lab<Self>;
    const NAVY: Lab<Self>;
    const INDIGO: Lab<Self>;
    const VIOLET: Lab<Self>;
    const GOLD: Lab<Self>;
    const SILVER: Lab<Self>;
    const BEIGE: Lab<Self>;
    const IVORY: Lab<Self>;
    const PEACH: Lab<Self>;
    const CHOCOLATE: Lab<Self>;
}

impl Float for f32 {
    const HALF: Self = 0.5;
    const EPSILON: Self = f32::EPSILON;
    const INFINITY: Self = f32::INFINITY;
    const TWO: Self = 2.0;
    const PI: Self = core::f32::consts::PI;
    const SQRT140050: Self = f32::from_bits(0x43bb1dc4);
    const TWO_FIVE_FIVE: Self = 255.0;
    const HALF_SQRT140050: Self = Self::SQRT140050 * Self::HALF;
    const FLOYD_STEINBERG: [DitherWeight<Self>; 4] = [
        DitherWeight {
            pos: Point { x: 1, y: 0 },
            weight: 7.0 / 16.0,
        },
        DitherWeight {
            pos: Point { x: -1, y: 1 },
            weight: 3.0 / 16.0,
        },
        DitherWeight {
            pos: Point { x: 0, y: 1 },
            weight: 5.0 / 16.0,
        },
        DitherWeight {
            pos: Point { x: 1, y: 1 },
            weight: 1.0 / 16.0,
        },
    ];
    const MINUS_HUNDRED_TWENTY_EIGHT: Self = -128.0;
    const HUNDRED_TWENTY_SEVEN: Self = 127.0;
    const HUNDRED: Self = 100.0;

    const RED: Lab<Self> = Lab::new(53.23, 80.09, 67.2);
    const GREEN: Lab<Self> = Lab::new(87.73, -86.18, 83.18);
    const BLUE: Lab<Self> = Lab::new(32.3, 79.2, -107.86);
    const YELLOW: Lab<Self> = Lab::new(97.14, -21.56, 94.48);
    const BLACK: Lab<Self> = Lab::new(0.0, 0.0, 0.0);
    const WHITE: Lab<Self> = Lab::new(100.0, 0.0, 0.0);
    const GRAY: Lab<Self> = Lab::new(53.59, 0.0, 0.0);
    const ORANGE: Lab<Self> = Lab::new(60.32, 56.04, 60.35);
    const PURPLE: Lab<Self> = Lab::new(29.85, 58.71, -37.46);
    const BROWN: Lab<Self> = Lab::new(45.06, 33.92, 17.47);
    const PINK: Lab<Self> = Lab::new(76.91, 16.65, 6.48);
    const CYAN: Lab<Self> = Lab::new(91.11, -48.09, -14.13);
    const MAGENTA: Lab<Self> = Lab::new(60.3, 98.25, -60.67);
    const LIME: Lab<Self> = Lab::new(87.73, -86.18, 83.18);
    const TEAL: Lab<Self> = Lab::new(55.14, -35.5, -25.02);
    const NAVY: Lab<Self> = Lab::new(26.99, 33.99, -68.2);
    const INDIGO: Lab<Self> = Lab::new(32.25, 54.47, -29.97);
    const VIOLET: Lab<Self> = Lab::new(68.67, 62.28, -64.23);
    const GOLD: Lab<Self> = Lab::new(84.34, 23.83, 77.21);
    const SILVER: Lab<Self> = Lab::new(76.88, -3.19, 1.45);
    const BEIGE: Lab<Self> = Lab::new(94.12, 2.13, 14.19);
    const IVORY: Lab<Self> = Lab::new(98.39, 0.53, 8.75);
    const PEACH: Lab<Self> = Lab::new(76.96, 22.98, 32.48);
    const CHOCOLATE: Lab<Self> = Lab::new(45.13, 26.95, 22.05);
}

impl Float for f64 {
    const HALF: Self = 0.5;
    const EPSILON: Self = f64::EPSILON;
    const INFINITY: Self = f64::INFINITY;
    const TWO: Self = 2.0;
    const PI: Self = core::f64::consts::PI;
    const SQRT140050: Self = f64::from_bits(0x407763b88446ac1c);
    const TWO_FIVE_FIVE: Self = 255.0;
    const HALF_SQRT140050: Self = Self::SQRT140050 * Self::HALF;
    const FLOYD_STEINBERG: [DitherWeight<Self>; 4] = [
        DitherWeight {
            pos: Point { x: 1, y: 0 },
            weight: 7.0 / 16.0,
        },
        DitherWeight {
            pos: Point { x: -1, y: 1 },
            weight: 3.0 / 16.0,
        },
        DitherWeight {
            pos: Point { x: 0, y: 1 },
            weight: 5.0 / 16.0,
        },
        DitherWeight {
            pos: Point { x: 1, y: 1 },
            weight: 1.0 / 16.0,
        },
    ];
    const MINUS_HUNDRED_TWENTY_EIGHT: Self = -128.0;
    const HUNDRED_TWENTY_SEVEN: Self = 127.0;
    const HUNDRED: Self = 100.0;

    const RED: Lab<Self> = Lab::new(53.23, 80.09, 67.2);
    const GREEN: Lab<Self> = Lab::new(87.73, -86.18, 83.18);
    const BLUE: Lab<Self> = Lab::new(32.3, 79.2, -107.86);
    const YELLOW: Lab<Self> = Lab::new(97.14, -21.56, 94.48);
    const BLACK: Lab<Self> = Lab::new(0.0, 0.0, 0.0);
    const WHITE: Lab<Self> = Lab::new(100.0, 0.0, 0.0);
    const GRAY: Lab<Self> = Lab::new(53.59, 0.0, 0.0);
    const ORANGE: Lab<Self> = Lab::new(60.32, 56.04, 60.35);
    const PURPLE: Lab<Self> = Lab::new(29.85, 58.71, -37.46);
    const BROWN: Lab<Self> = Lab::new(45.06, 33.92, 17.47);
    const PINK: Lab<Self> = Lab::new(76.91, 16.65, 6.48);
    const CYAN: Lab<Self> = Lab::new(91.11, -48.09, -14.13);
    const MAGENTA: Lab<Self> = Lab::new(60.3, 98.25, -60.67);
    const LIME: Lab<Self> = Lab::new(87.73, -86.18, 83.18);
    const TEAL: Lab<Self> = Lab::new(55.14, -35.5, -25.02);
    const NAVY: Lab<Self> = Lab::new(26.99, 33.99, -68.2);
    const INDIGO: Lab<Self> = Lab::new(32.25, 54.47, -29.97);
    const VIOLET: Lab<Self> = Lab::new(68.67, 62.28, -64.23);
    const GOLD: Lab<Self> = Lab::new(84.34, 23.83, 77.21);
    const SILVER: Lab<Self> = Lab::new(76.88, -3.19, 1.45);
    const BEIGE: Lab<Self> = Lab::new(94.12, 2.13, 14.19);
    const IVORY: Lab<Self> = Lab::new(98.39, 0.53, 8.75);
    const PEACH: Lab<Self> = Lab::new(76.96, 22.98, 32.48);
    const CHOCOLATE: Lab<Self> = Lab::new(45.13, 26.95, 22.05);
}
