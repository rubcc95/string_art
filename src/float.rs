use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

use num_traits::{ConstOne, ConstZero};

use crate::{ditherer::DitherWeight, geometry::Point};

pub trait Float:
    'static
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
}
