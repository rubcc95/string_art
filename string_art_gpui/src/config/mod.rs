//! Persistent application settings.
//!
//! The configuration lives outside the repository: in the platform's config
//! directory on native builds and in `localStorage` on the web. It is read at
//! startup and saved by [`Config::save`] when a setting changes. There is no
//! settings UI yet, but the file can be edited by hand.

#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod web;

#[cfg(not(target_family = "wasm"))]
use native as platform;
#[cfg(target_family = "wasm")]
use web as platform;

use std::fmt;

const CURRENT_VERSION: u32 = 1;

/// Everything the application persists between runs.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    /// BCP 47 language tag. `None` asks for the system language.
    #[serde(default)]
    pub language: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            language: None,
        }
    }
}

fn default_version() -> u32 {
    CURRENT_VERSION
}

impl Config {
    /// Reads the configuration, falling back to defaults when the file is
    /// missing, unreadable or malformed.
    pub fn load() -> Self {
        let Some(contents) = platform::read() else {
            return Self::default();
        };

        match serde_json::from_str::<Self>(&contents) {
            Ok(config) => {
                if config.version > CURRENT_VERSION {
                    warn(&format!(
                        "config version {} is newer than the supported version {CURRENT_VERSION}",
                        config.version
                    ));
                }
                config
            }
            Err(error) => {
                warn(&format!("ignoring malformed configuration: {error}"));
                Self::default()
            }
        }
    }

    /// Writes the configuration. Unused until a settings UI exists, but kept
    /// ready so the persistence layer is complete.
    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), ConfigError> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|error| ConfigError::new(error.to_string()))?;
        platform::write(&contents)
    }

    /// Where the configuration is stored, for diagnostics and documentation.
    #[allow(dead_code)]
    pub fn path() -> Option<String> {
        platform::path()
    }

    /// The locale this configuration asks for, independent of the system.
    #[allow(dead_code)]
    pub fn configured_locale(&self) -> Option<&'static str> {
        self.language
            .as_deref()
            .and_then(crate::i18n::normalize_tag)
    }
}

/// A failure while writing the configuration.
#[derive(Debug)]
pub struct ConfigError(String);

impl ConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

/// Reports a configuration problem without aborting startup.
fn warn(message: &str) {
    #[cfg(not(target_family = "wasm"))]
    eprintln!("string_art: {message}");
    #[cfg(target_family = "wasm")]
    {
        let message = wasm_bindgen::JsValue::from_str(&format!("string_art: {message}"));
        web_sys::console::warn_1(&message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_language_means_automatic() {
        let config: Config = serde_json::from_str("{}").expect("empty config should parse");
        assert_eq!(config.version, CURRENT_VERSION);
        assert_eq!(config.language, None);
        assert_eq!(config.configured_locale(), None);
    }

    #[test]
    fn reads_a_language_tag() {
        let config: Config =
            serde_json::from_str(r#"{"language":"es-ES"}"#).expect("config should parse");
        assert_eq!(config.configured_locale(), Some("es"));
    }

    #[test]
    fn ignores_unknown_languages() {
        let config: Config =
            serde_json::from_str(r#"{"language":"xx"}"#).expect("config should parse");
        assert_eq!(config.configured_locale(), None);
    }
}
