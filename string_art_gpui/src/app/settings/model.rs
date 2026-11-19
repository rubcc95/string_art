//! Computation settings model, independent from the UI.

use rust_i18n::t;

/// Smallest physical size of the work, in centimetres.
const MIN_SIZE_CM: f64 = 10.;
/// Largest physical size of the work, in centimetres.
const MAX_SIZE_CM: f64 = 1000.;
/// Physical width of the work when the ellipse has not been resized yet.
const DEFAULT_WIDTH_CM: f64 = 30.;
/// Default amount of energy removed when a thread passes over a pixel.
const DEFAULT_DECAY: f32 = 0.9;

/// The settings a computation is started with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Settings {
    pub nail_count: usize,
    pub min_nail_distance: usize,
    pub nail_radius_px: f32,
    pub thread_width_px: f32,
    pub decay: f32,
    pub width_cm: f32,
    pub height_cm: f32,
    /// Height divided by width of the ellipse, in image pixels. The physical
    /// size of the work follows it so the drawing is never stretched.
    aspect: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            nail_count: 512,
            min_nail_distance: 20,
            nail_radius_px: 2.,
            thread_width_px: 1.,
            decay: DEFAULT_DECAY,
            width_cm: DEFAULT_WIDTH_CM as f32,
            height_cm: DEFAULT_WIDTH_CM as f32,
            aspect: 1.,
        }
    }
}

impl Settings {
    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    /// Sets the aspect ratio of the ellipse and adapts the height to it,
    /// keeping the width fixed. If the resulting height would fall outside the
    /// accepted range, the width is adjusted instead so the proportion is
    /// preserved.
    pub fn set_ellipse_aspect(&mut self, aspect: f64) {
        if !aspect.is_finite() || aspect <= 0. {
            return;
        }
        // Outside this interval there is no pair of dimensions inside
        // `MIN_SIZE_CM..=MAX_SIZE_CM` with the requested proportion.
        let aspect = aspect.clamp(MIN_SIZE_CM / MAX_SIZE_CM, MAX_SIZE_CM / MIN_SIZE_CM) as f32;
        self.aspect = aspect;

        let aspect = f64::from(aspect);
        let width = f64::from(self.width_cm);
        let height = width * aspect;
        if height < MIN_SIZE_CM {
            let width = round_tenth(MIN_SIZE_CM / aspect).clamp(MIN_SIZE_CM, MAX_SIZE_CM);
            self.width_cm = width as f32;
            self.height_cm = round_tenth(width * aspect) as f32;
        } else if height > MAX_SIZE_CM {
            let width = round_tenth(MAX_SIZE_CM / aspect).clamp(MIN_SIZE_CM, MAX_SIZE_CM);
            self.width_cm = width as f32;
            self.height_cm = round_tenth(width * aspect) as f32;
        } else {
            self.height_cm = round_tenth(height) as f32;
        }
    }

    /// Initialises the proportion from the size of the imported image.
    pub fn set_image_aspect(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.set_ellipse_aspect(f64::from(height) / f64::from(width));
        }
    }

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
            SettingField::Decay => self.decay = round_hundredth(value) as f32,
            SettingField::Width => self.set_width(value),
            SettingField::Height => self.set_height(value),
        }
    }

    fn set_width(&mut self, value: f64) {
        let aspect = f64::from(self.aspect);
        // Keep the derived height inside the accepted range too.
        let min = (MIN_SIZE_CM / aspect).max(MIN_SIZE_CM);
        let max = (MAX_SIZE_CM / aspect).min(MAX_SIZE_CM);
        let width = value.clamp(min, max);
        self.width_cm = round_tenth(width) as f32;
        self.height_cm = round_tenth(width * aspect) as f32;
    }

    fn set_height(&mut self, value: f64) {
        let aspect = f64::from(self.aspect);
        // Keep the derived width inside the accepted range too.
        let min = (MIN_SIZE_CM * aspect).max(MIN_SIZE_CM);
        let max = (MAX_SIZE_CM * aspect).min(MAX_SIZE_CM);
        let height = value.clamp(min, max);
        self.height_cm = round_tenth(height) as f32;
        self.width_cm = round_tenth(height / aspect) as f32;
    }
}

fn round_tenth(value: f64) -> f64 {
    (value * 10.).round() / 10.
}

fn round_hundredth(value: f64) -> f64 {
    (value * 100.).round() / 100.
}

/// One of the values edited in the settings view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingField {
    Nails,
    MinDistance,
    Radius,
    ThreadWidth,
    Decay,
    Width,
    Height,
}

impl SettingField {
    pub const ALL: [Self; 7] = [
        Self::Nails,
        Self::MinDistance,
        Self::Radius,
        Self::ThreadWidth,
        Self::Decay,
        Self::Width,
        Self::Height,
    ];

    pub fn label(self) -> String {
        match self {
            Self::Nails => t!("settings.nails.label"),
            Self::MinDistance => t!("settings.min_distance.label"),
            Self::Radius => t!("settings.radius.label"),
            Self::ThreadWidth => t!("settings.thread_width.label"),
            Self::Decay => t!("settings.decay.label"),
            Self::Width => t!("settings.width.label"),
            Self::Height => t!("settings.height.label"),
        }
        .to_string()
    }

    pub fn hint(self) -> String {
        match self {
            Self::Nails => t!("settings.nails.hint"),
            Self::MinDistance => t!("settings.min_distance.hint"),
            Self::Radius => t!("settings.radius.hint"),
            Self::ThreadWidth => t!("settings.thread_width.hint"),
            Self::Decay => t!("settings.decay.hint"),
            Self::Width => t!("settings.width.hint"),
            Self::Height => t!("settings.height.hint"),
        }
        .to_string()
    }

    pub fn suffix(self) -> String {
        match self {
            Self::Nails | Self::MinDistance | Self::Decay => String::new(),
            Self::Radius | Self::ThreadWidth => t!("units.px").to_string(),
            Self::Width | Self::Height => t!("units.cm").to_string(),
        }
    }

    pub fn decimals(self) -> u8 {
        match self {
            Self::Nails | Self::MinDistance => 0,
            Self::Decay => 2,
            Self::Radius | Self::ThreadWidth | Self::Width | Self::Height => 1,
        }
    }

    pub fn step(self) -> f64 {
        match self {
            Self::Nails | Self::MinDistance => 1.,
            Self::Decay => 0.01,
            Self::Radius | Self::ThreadWidth | Self::Width | Self::Height => 0.1,
        }
    }

    pub fn value(self, settings: &Settings) -> f64 {
        match self {
            Self::Nails => settings.nail_count as f64,
            Self::MinDistance => settings.min_nail_distance as f64,
            Self::Radius => f64::from(settings.nail_radius_px),
            Self::ThreadWidth => f64::from(settings.thread_width_px),
            Self::Decay => f64::from(settings.decay),
            Self::Width => f64::from(settings.width_cm),
            Self::Height => f64::from(settings.height_cm),
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
            Self::Decay => (0., 1.),
            Self::Width | Self::Height => (MIN_SIZE_CM, MAX_SIZE_CM),
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
        settings.apply(SettingField::Decay, 9.);
        assert_eq!(settings.decay, 1.);
    }

    #[test]
    fn default_physical_size_matches_the_image_aspect() {
        let mut settings = Settings::default();
        settings.set_image_aspect(64, 32);
        assert!((settings.width_cm - 30.).abs() < 1e-4);
        assert!((settings.height_cm - 15.).abs() < 1e-4);
        assert!((settings.aspect() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn editing_one_dimension_keeps_the_aspect() {
        let mut settings = Settings::default();
        settings.set_ellipse_aspect(0.5);

        settings.apply(SettingField::Width, 60.);
        assert!((settings.width_cm - 60.).abs() < 1e-4);
        assert!((settings.height_cm - 30.).abs() < 1e-4);

        settings.apply(SettingField::Height, 20.);
        assert!((settings.width_cm - 40.).abs() < 1e-4);
        assert!((settings.height_cm - 20.).abs() < 1e-4);
    }

    #[test]
    fn resizing_the_ellipse_changes_the_height_not_the_width() {
        let mut settings = Settings::default();
        settings.set_ellipse_aspect(0.5);
        settings.set_ellipse_aspect(0.75);
        assert!((settings.width_cm - 30.).abs() < 1e-4);
        assert!((settings.height_cm - 22.5).abs() < 1e-4);
    }

    #[test]
    fn physical_dimensions_stay_inside_their_range_without_losing_the_aspect() {
        let mut settings = Settings::default();
        settings.set_ellipse_aspect(2.);

        settings.apply(SettingField::Width, 1000.);
        assert!((settings.width_cm - 500.).abs() < 1e-4);
        assert!((settings.height_cm - 1000.).abs() < 1e-4);

        settings.apply(SettingField::Height, 1000.);
        assert!((settings.width_cm - 500.).abs() < 1e-4);
        assert!((settings.height_cm - 1000.).abs() < 1e-4);
    }
}
