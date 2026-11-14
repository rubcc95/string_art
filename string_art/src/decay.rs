use crate::num::Frac;

/// How a pixel's energy is reduced when a thread passes over it.
#[derive(Clone, Copy, Debug)]
pub enum Decay {
    /// Subtracts a fixed amount. Saturates at zero.
    Flat(Frac),
    /// Keeps a percentage of the energy (between 0 and 1).
    Percentage(Frac),
}

impl Decay {
    pub fn compute(&self, weight: Frac) -> Frac {
        match *self {
            Self::Flat(amount) => {
                if weight > amount {
                    weight - amount
                } else {
                    Frac::ZERO
                }
            }
            Self::Percentage(ratio) => ratio * weight,
        }
    }
}
