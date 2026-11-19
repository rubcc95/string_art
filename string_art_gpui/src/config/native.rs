//! Native configuration storage: `<config dir>/string_art/config.json`.

use std::io::ErrorKind;
use std::path::PathBuf;

use directories::ProjectDirs;

use super::{ConfigError, warn};

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "rubcc95", "string_art")
}

fn config_path() -> Option<PathBuf> {
    Some(project_dirs()?.config_dir().join("config.json"))
}

pub(super) fn path() -> Option<String> {
    config_path().map(|path| path.display().to_string())
}

pub(super) fn read() -> Option<String> {
    let path = config_path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => Some(contents),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => {
            warn(&format!("could not read {}: {error}", path.display()));
            None
        }
    }
}

pub(super) fn write(contents: &str) -> Result<(), ConfigError> {
    let path = config_path()
        .ok_or_else(|| ConfigError::new("the platform has no configuration directory"))?;
    let parent = path
        .parent()
        .ok_or_else(|| ConfigError::new("the configuration path has no parent directory"))?;
    std::fs::create_dir_all(parent).map_err(|error| {
        ConfigError::new(format!("could not create {}: {error}", parent.display()))
    })?;

    // Write next to the target and rename, so a crash never leaves a
    // half-written configuration behind.
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, contents).map_err(|error| {
        ConfigError::new(format!("could not write {}: {error}", temporary.display()))
    })?;
    std::fs::rename(&temporary, &path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        ConfigError::new(format!("could not replace {}: {error}", path.display()))
    })
}
