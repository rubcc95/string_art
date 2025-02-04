use std::{
    fmt::Display,
    ops::{AddAssign, DivAssign, MulAssign, SubAssign},
};

use num_traits::{AsPrimitive, ConstOne, ConstZero};

use crate::{image::dither::Weight, geometry::Point};

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
    + AsPrimitive<Self>
{
    const HALF: Self;
    const INFINITY: Self;
    const EPSILON: Self;
    const TWO: Self;
    const THREE: Self; // max euclidean distance for a LAB color
    const TWO_FIVE_FIVE: Self;
    const FLOYD_STEINBERG: [Weight<Self>; 4];

    fn min(self, other: Self) -> Self;

    const FRAC_PI_4: Self;
    const FRAC_PI_2: Self;
    const FRAC_3PI_4: Self;
    const PI: Self;
    const FRAC_5PI_4: Self;
    const FRAC_3PI_2: Self;
    const FRAC_7PI_4: Self;
    const PI2: Self;
}

impl Float for f32 {
    const HALF: Self = 0.5;
    const INFINITY: Self = f32::INFINITY;
    const EPSILON: Self = f32::EPSILON;
    const TWO: Self = 2.0;
    const THREE: Self = 3.0; //f32::from_bits(0x43bb1dc4);
    const TWO_FIVE_FIVE: Self = 255.0;
    const FLOYD_STEINBERG: [Weight<Self>; 4] = [
        Weight {
            pos: Point { x: 1, y: 0 },
            weight: 7.0 / 16.0,
        },
        Weight {
            pos: Point { x: -1, y: 1 },
            weight: 3.0 / 16.0,
        },
        Weight {
            pos: Point { x: 0, y: 1 },
            weight: 5.0 / 16.0,
        },
        Weight {
            pos: Point { x: 1, y: 1 },
            weight: 1.0 / 16.0,
        },
    ];

    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    const FRAC_PI_4: Self = core::f32::consts::FRAC_PI_4;
    const FRAC_PI_2: Self = core::f32::consts::FRAC_PI_2;
    const FRAC_3PI_4: Self = 3.0 * core::f32::consts::FRAC_PI_4;
    const PI: Self = core::f32::consts::PI;
    const FRAC_5PI_4: Self = 5.0 * core::f32::consts::FRAC_PI_4;
    const FRAC_3PI_2: Self = 3.0 * core::f32::consts::FRAC_PI_2;
    const FRAC_7PI_4: Self = 7.0 * core::f32::consts::FRAC_PI_4;
    const PI2: Self = 2.0 * core::f32::consts::PI;
}

impl Float for f64 {
    const HALF: Self = 0.5;
    const INFINITY: Self = f64::INFINITY;
    const EPSILON: Self = f64::EPSILON;
    const TWO: Self = 2.0;
    const THREE: Self = 3.0; //f64::from_bits(0x407763b88446ac1c);
    const TWO_FIVE_FIVE: Self = 255.0;
    const FLOYD_STEINBERG: [Weight<Self>; 4] = [
        Weight {
            pos: Point { x: 1, y: 0 },
            weight: 7.0 / 16.0,
        },
        Weight {
            pos: Point { x: -1, y: 1 },
            weight: 3.0 / 16.0,
        },
        Weight {
            pos: Point { x: 0, y: 1 },
            weight: 5.0 / 16.0,
        },
        Weight {
            pos: Point { x: 1, y: 1 },
            weight: 1.0 / 16.0,
        },
    ];

    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    const FRAC_PI_4: Self = core::f64::consts::FRAC_PI_4;
    const FRAC_PI_2: Self = core::f64::consts::FRAC_PI_2;
    const FRAC_3PI_4: Self = 3.0 * core::f64::consts::FRAC_PI_4;
    const PI: Self = core::f64::consts::PI;
    const FRAC_5PI_4: Self = 5.0 * core::f64::consts::FRAC_PI_4;
    const FRAC_3PI_2: Self = 3.0 * core::f64::consts::FRAC_PI_2;
    const FRAC_7PI_4: Self = 7.0 * core::f64::consts::FRAC_PI_4;
    const PI2: Self = 2.0 * core::f64::consts::PI;
}
