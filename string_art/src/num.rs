//! Core numeric types.
//!
//! There is no configuration: all energy computation uses 64-bit fixed point.

use fixed::types::{U0F32, U32F32};

/// Remaining energy of a pixel. Range `[0, 1)`.
pub type Frac = U0F32;

/// Energy accumulator: 32 integer bits and 32 fractional bits.
///
/// Allows adding many `Frac` values without overflowing before dividing by the
/// number of pixels.
pub type Scalar = U32F32;

/// 8-bit subpixel to `Frac`, replicating the byte across all fractional
/// bits (0 to 0, 128 to ~0.502, 255 to ~1.0).
pub(crate) fn frac_from_u8(value: u8) -> Frac {
    Frac::from_bits(u32::from(value) * 0x0101_0101)
}

/// `Frac` to `Scalar`, so it can accumulate without leaving the range.
pub(crate) fn scalar_from_frac(frac: Frac) -> Scalar {
    Scalar::from_bits(u64::from(frac.to_bits()))
}
