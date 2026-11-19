//! Web configuration storage: a single `localStorage` entry.
//!
//! `localStorage` is unavailable in some contexts (for example, private mode
//! with storage disabled), so every operation degrades to the defaults.

use super::ConfigError;

const KEY: &str = "string_art.config";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub(super) fn path() -> Option<String> {
    Some(format!("localStorage[{KEY}]"))
}

pub(super) fn read() -> Option<String> {
    storage()?.get_item(KEY).ok().flatten()
}

pub(super) fn write(contents: &str) -> Result<(), ConfigError> {
    let storage = storage().ok_or_else(|| ConfigError::new("localStorage is not available"))?;
    storage
        .set_item(KEY, contents)
        .map_err(|_| ConfigError::new("could not write the configuration to localStorage"))
}
