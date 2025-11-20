use num_traits::{NumAssignOps, NumOps, SaturatingSub};

pub type Fixed16 = fixed::types::U8F8;
pub type Fixed32 = fixed::types::U16F16;
pub type Fixed64 = fixed::types::U32F32;
pub type Fixed128 = fixed::types::U64F64;

pub type Frac8 = fixed::types::U0F8;
pub type Frac16 = fixed::types::U0F16;
pub type Frac32 = fixed::types::U0F32;
pub type Frac64 = fixed::types::U0F64;

pub trait Scalar
where
    Self: PartialOrd,
    Self: NumOps + NumAssignOps,
    Self: Copy + Send + Sync + 'static,
{
    const ZERO: Self;
}

macro_rules! scalar_impl {
        ($($t:ty),+; $zero:expr) => {
            $(
                impl Scalar for $t {
                    const ZERO: Self = $zero;
                }
            )+
        };
    }

scalar_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128; 0);
scalar_impl!(f32, f64; 0.0);
scalar_impl!(Fixed16, Fixed32, Fixed64, Fixed128, Frac8, Frac16, Frac32, Frac64; Self::ZERO);

pub trait Integer: Scalar + Ord {
    const ONE: Self;
}

macro_rules! integer_impl {
        ($($t:ty),+; $one:expr) => {
            $(
                impl Integer for $t {
                    const ONE: Self = $one;
                }
            )+
        };
    }

integer_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128; 1);

pub trait Frac: Scalar + Ord + SaturatingSub {
    type Bits: Integer;
    type Fixed: Fixed<Frac = Self>;

    fn from_bits(bits: Self::Bits) -> Self;

    fn to_bits(self) -> Self::Bits;
}

macro_rules! frac_impl {
    ($U:ty, $Bits:ty, $Fixed:ty) => {
        impl Frac for $U {
            type Bits = $Bits;
            type Fixed = $Fixed;

            fn from_bits(bits: Self::Bits) -> Self {
                <$U>::from_bits(bits)
            }

            fn to_bits(self) -> Self::Bits {
                self.to_bits()
            }
        }
    };
}

frac_impl!(Frac8, u8, Fixed16);
frac_impl!(Frac16, u16, Fixed32);
frac_impl!(Frac32, u32, Fixed64);
frac_impl!(Frac64, u64, Fixed128);

pub trait Fixed: Scalar + Ord {
    type Int: Integer;
    type Frac: Frac;

    fn from_int(int: Self::Int) -> Self;
    fn from_frac(frac: Self::Frac) -> Self;
    fn int(self) -> Self::Int;
    fn frac(self) -> Self::Frac;
}

macro_rules! impl_fixed {
    ($Fixed:ident, $Bits:ty, $Int:ty, $Frac:ty) => {
        impl Fixed for $Fixed {
            type Int = $Int;
            type Frac = $Frac;

            fn from_int(int: Self::Int) -> Self {
                $Fixed::from_bits((int as $Bits) << <$Int>::BITS)
            }

            fn from_frac(frac: Self::Frac) -> Self {
                Self::from_bits(frac.to_bits() as _)
            }

            fn int(self) -> Self::Int {
                (self.to_bits() >> <$Int>::BITS) as $Int
            }

            fn frac(self) -> Self::Frac {
                <$Frac>::from_bits(self.to_bits() as _)
            }
        }
    };
}

impl_fixed!(Fixed16, u16, u8, Frac8);
impl_fixed!(Fixed32, u32, u16, Frac16);
impl_fixed!(Fixed64, u64, u32, Frac32);
impl_fixed!(Fixed128, u128, u64, Frac64);
