use crate::num::Frac;

/// Cómo se reduce la energía de un píxel cuando pasa un hilo por encima.
#[derive(Clone, Copy, Debug)]
pub enum Decay {
    /// Resta una cantidad fija. Se satura en cero.
    Flat(Frac),
    /// Conserva un porcentaje de la energía (entre 0 y 1).
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
