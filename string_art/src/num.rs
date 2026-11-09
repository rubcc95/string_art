//! Tipos numéricos del core.
//!
//! No hay configuración: todo el cálculo de energía va en punto fijo de 64 bits.

use fixed::types::{U0F32, U32F32};

/// Energía restante de un píxel. Rango `[0, 1)`.
pub type Frac = U0F32;

/// Acumulador de energía: 32 bits enteros y 32 fraccionarios.
///
/// Permite sumar muchos `Frac` sin desbordar antes de dividir por el número
/// de píxeles.
pub type Scalar = U32F32;

/// Subpíxel de 8 bits a `Frac`, replicando el byte por todos los bits
/// fraccionarios (0 a 0, 128 a ~0,502, 255 a ~1,0).
pub fn frac_from_u8(value: u8) -> Frac {
    Frac::from_bits(u32::from(value) * 0x0101_0101)
}

/// `Frac` a `Scalar`, para poder acumular sin salir del rango.
pub fn scalar_from_frac(frac: Frac) -> Scalar {
    Scalar::from_bits(u64::from(frac.to_bits()))
}
