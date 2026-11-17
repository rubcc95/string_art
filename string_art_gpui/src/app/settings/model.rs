//! Computation settings model, independent from the UI.

/// The settings a computation is started with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Settings {
    pub nail_count: usize,
    pub min_nail_distance: usize,
    pub nail_radius_px: f32,
    pub thread_width_px: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            nail_count: 512,
            min_nail_distance: 20,
            nail_radius_px: 2.,
            thread_width_px: 1.,
        }
    }
}

impl Settings {
    /// Applies a value to one field, clamped to its valid range. Changing the
    /// nail count also keeps the minimum distance within the bound accepted by
    /// [`string_art::Ellipse`].
    pub fn apply(&mut self, field: SettingField, value: f64) {
        let (min, max) = field.range(self);
        let value = value.clamp(min, max);
        match field {
            SettingField::Nails => {
                self.nail_count = value.round() as usize;
                let max_distance = self.nail_count.saturating_sub(1) / 2;
                self.min_nail_distance = self.min_nail_distance.clamp(1, max_distance.max(1));
            }
            SettingField::MinDistance => {
                self.min_nail_distance = value.round() as usize;
            }
            SettingField::Radius => self.nail_radius_px = round_tenth(value) as f32,
            SettingField::ThreadWidth => self.thread_width_px = round_tenth(value) as f32,
        }
    }
}

fn round_tenth(value: f64) -> f64 {
    (value * 10.).round() / 10.
}

/// One of the values edited in the settings view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingField {
    Nails,
    MinDistance,
    Radius,
    ThreadWidth,
}

impl SettingField {
    pub const ALL: [Self; 4] = [
        Self::Nails,
        Self::MinDistance,
        Self::Radius,
        Self::ThreadWidth,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Nails => "Número de clavos",
            Self::MinDistance => "Distancia mínima",
            Self::Radius => "Radio del clavo",
            Self::ThreadWidth => "Ancho del hilo",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Nails => "Clavos repartidos por el borde de la imagen",
            Self::MinDistance => "Mínimo de clavos entre dos anclajes",
            Self::Radius => "Radio de cada clavo, en píxeles",
            Self::ThreadWidth => "Grosor del hilo, en píxeles",
        }
    }

    pub fn suffix(self) -> &'static str {
        match self {
            Self::Nails | Self::MinDistance => "",
            Self::Radius | Self::ThreadWidth => "px",
        }
    }

    pub fn is_integer(self) -> bool {
        matches!(self, Self::Nails | Self::MinDistance)
    }

    pub fn decimals(self) -> u8 {
        if self.is_integer() { 0 } else { 1 }
    }

    pub fn step(self) -> f64 {
        if self.is_integer() { 1. } else { 0.1 }
    }

    pub fn value(self, settings: &Settings) -> f64 {
        match self {
            Self::Nails => settings.nail_count as f64,
            Self::MinDistance => settings.min_nail_distance as f64,
            Self::Radius => f64::from(settings.nail_radius_px),
            Self::ThreadWidth => f64::from(settings.thread_width_px),
        }
    }

    pub fn range(self, settings: &Settings) -> (f64, f64) {
        match self {
            Self::Nails => (32., 4096.),
            Self::MinDistance => {
                let max = (settings.nail_count.saturating_sub(1) / 2).max(1);
                (1., max as f64)
            }
            Self::Radius | Self::ThreadWidth => (0.1, 8.),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowering_the_nail_count_keeps_the_minimum_distance_valid() {
        let mut settings = Settings::default();
        settings.apply(SettingField::Nails, 32.);
        assert_eq!(settings.nail_count, 32);
        assert_eq!(settings.min_nail_distance, 15);
    }

    #[test]
    fn values_are_clamped_and_rounded() {
        let mut settings = Settings::default();
        settings.apply(SettingField::Radius, 99.);
        assert_eq!(settings.nail_radius_px, 8.);
        settings.apply(SettingField::ThreadWidth, 0.03);
        assert_eq!(settings.thread_width_px, 0.1);
        settings.apply(SettingField::MinDistance, 0.);
        assert_eq!(settings.min_nail_distance, 1);
    }
}
