use crate::*;

pub trait IntoFrac<T> {
    fn into_frac(self) -> T;
}

pub trait FromFrac<T> {
    fn from_frac(int: T) -> Self;
}

impl<T: FromFrac<U>, U> IntoFrac<T> for U {
    fn into_frac(self) -> T {
        T::from_frac(self)
    }
}

macro_rules! impl_scaled_replicate {
    ($from:ty => $to:ty) => {
        impl FromFrac<$from> for $to {
            fn from_frac(int: $from) -> Self {
                <$to>::from_bits(int)
            }
        }
    };

    ($from:ty => $to:ty, $replicate:expr) => {
        impl FromFrac<$from> for $to {
            fn from_frac(int: $from) -> Self {
                <$to>::from_bits((int as <$to as Fixed>::Bits) * $replicate)
            }
        }
    };

    ($from:ty => $to:ty, d $shift:expr) => {
        impl FromFrac<$from> for $to {
            fn from_frac(int: $from) -> Self {
                <$to>::from_bits((int >> $shift) as <$to as Fixed>::Bits)
            }
        }
    };
}

impl_scaled_replicate!(u8 => Frac8);
impl_scaled_replicate!(u8 => Frac16, 0x0101);
impl_scaled_replicate!(u8 => Frac32, 0x01010101);
impl_scaled_replicate!(u8 => Frac64, 0x0101010101010101);

impl_scaled_replicate!(u16 => Frac16);
impl_scaled_replicate!(u16 => Frac8, d 8);
impl_scaled_replicate!(u16 => Frac32, 0x00010001);
impl_scaled_replicate!(u16 => Frac64, 0x0001000100010001);

impl_scaled_replicate!(u32 => Frac32);
impl_scaled_replicate!(u32 => Frac8, d 24);
impl_scaled_replicate!(u32 => Frac16, d 16);
impl_scaled_replicate!(u32 => Frac64, 0x0000000100000001);

impl_scaled_replicate!(u64 => Frac64);
impl_scaled_replicate!(u64 => Frac8, d 56);
impl_scaled_replicate!(u64 => Frac16, d 48);
impl_scaled_replicate!(u64 => Frac32, d 32);

impl_scaled_replicate!(u128 => Frac8, d 120);
impl_scaled_replicate!(u128 => Frac16, d 112);
impl_scaled_replicate!(u128 => Frac32, d 96);
impl_scaled_replicate!(u128 => Frac64, d 64);
