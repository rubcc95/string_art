use super::*;
#[derive(Clone, Copy, PartialEq)]
pub struct ComputationSettings {
    pub decay_mode: DecayFnMode,
    pub decay_flat: f32,
    pub decay_per: f32,
    pub resolution: u32,
    pub precision: input::computation_settings::Precision,
}

impl ComputationSettings {
    pub fn set(&mut self, input: input::ComputationSettings) {
        match input.decay_fn {
            input::computation_settings::DecayFn::Flat(flat) => {
                self.decay_mode = DecayFnMode::Flat;
                self.decay_flat = flat;
            }
            input::computation_settings::DecayFn::Percentage(per) => {
                self.decay_mode = DecayFnMode::Percentage;
                self.decay_per = per;
            }
        }
        self.resolution = input.resolution;
        self.precision = input.precision;
    }
}

impl Into<input::ComputationSettings> for ComputationSettings {
    fn into(self) -> input::ComputationSettings {
        input::ComputationSettings {
            decay_fn: match self.decay_mode {
                DecayFnMode::Flat => input::computation_settings::DecayFn::Flat(self.decay_flat),
                DecayFnMode::Percentage => {
                    input::computation_settings::DecayFn::Percentage(self.decay_per)
                }
            },
            resolution: self.resolution,
            precision: self.precision,
        }
    }
}

impl Default for ComputationSettings {
    fn default() -> Self {
        Self {
            decay_mode: Default::default(),
            decay_flat: 0.2,
            decay_per: 0.1,
            resolution: 1024,
            precision: Default::default(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum DecayFnMode {
    #[default]
    Flat,
    Percentage,
}
