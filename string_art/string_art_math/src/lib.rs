
use num_traits::*;
use fixed::types::*;
use std::fmt::*;

pub use fixed::traits::{Fixed, ToFixed};
pub use num_traits::ConstOne as _;
pub use num_traits::ConstZero as _;

pub type Frac8 = U0F8;
pub type Frac16 = U0F16;
pub type Frac32 = U0F32;
pub type Frac64 = U0F64;
pub type Scalar8 = U24F8;
pub type Scalar16 = U16F16;
pub type Scalar32 = U32F32;
pub type Scalar64 = U64F64;

mod convert;
pub use convert::*;

pub trait Frac
where
    Self: Copy + Eq + Ord,
    Self: Send + Sync + 'static,
    Self: NumAssignOps + NumOps + ConstZero,
    Self: Debug + Display,
{
}

impl<T> Frac for T
where
    T: Copy + Eq + Ord,
    T: Send + Sync + 'static,
    T: NumAssignOps + NumOps + ConstZero,
    T: Debug + Display,
{
}

pub trait Integer: Frac + ConstOne {}

impl<T> Integer for T where T: Frac + ConstOne {}

pub trait Scalar: Integer {
    type Frac: Frac;

    type Int: Integer;

    fn from_frac(frac: Self::Frac) -> Self;

    fn from_int(int: Self::Int) -> Self;

    fn frac(&self) -> Self::Frac;

    fn int(&self) -> Self::Int;
}

impl Scalar for Scalar8 {
    type Frac = Frac8;

    type Int = u32;

    fn from_frac(frac: Self::Frac) -> Self {
        U24F8::from_bits(frac.to_bits() as u32)
    }

    fn from_int(int: Self::Int) -> Self {
        U24F8::from_bits(int << 8)
    }

    fn frac(&self) -> Self::Frac {
        let bits = self.to_bits();
        U0F8::from_bits((bits & 0xFF) as u8)
    }

    fn int(&self) -> Self::Int {
        let bits = self.to_bits();
        (bits >> 8) as u32
    }
}

impl Scalar for Scalar16 {
    type Frac = Frac16;

    type Int = u16;

    fn from_frac(frac: Self::Frac) -> Self {
        U16F16::from_bits(frac.to_bits() as u32)
    }

    fn from_int(int: Self::Int) -> Self {
        U16F16::from_bits((int as u32) << 16)
    }

    fn frac(&self) -> Self::Frac {
        let bits = self.to_bits();
        U0F16::from_bits((bits & 0xFFFF) as u16)
    }

    fn int(&self) -> Self::Int {
        let bits = self.to_bits();
        (bits >> 16) as u16
    }
}

impl Scalar for Scalar32 {
    type Frac = Frac32;

    type Int = u32;

    fn from_frac(frac: Self::Frac) -> Self {
        U32F32::from_bits(frac.to_bits() as u64)
    }

    fn from_int(int: Self::Int) -> Self {
        U32F32::from_bits((int as u64) << 32)
    }

    fn frac(&self) -> Self::Frac {
        let bits = self.to_bits();
        U0F32::from_bits((bits & 0xFFFFFFFF) as u32)
    }

    fn int(&self) -> Self::Int {
        let bits = self.to_bits();
        (bits >> 8) as u32
    }
}

impl Scalar for Scalar64 {
    type Frac = Frac64;

    type Int = u64;

    fn from_frac(frac: Self::Frac) -> Self {
        U64F64::from_bits(frac.to_bits() as u128)
    }

    fn from_int(int: Self::Int) -> Self {
        U64F64::from_bits((int as u128) << 64)
    }

    fn frac(&self) -> Self::Frac {
        let bits = self.to_bits();
        U0F64::from_bits((bits & 0xFFFFFFFFFFFFFFFF) as u64)
    }

    fn int(&self) -> Self::Int {
        let bits = self.to_bits();
        (bits >> 8) as u64
    }
}
