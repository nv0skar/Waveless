// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// Retrieves the project's config from the `PROJECT_CONFIG`'s `OnceLock` or load it if not present.
pub fn project_config() -> Result<&'static project::Project> {
    match PROJECT_CONFIG.get() {
        Some(config) => Ok(config),
        None => {
            let config = load_config()?;
            PROJECT_CONFIG.set(config).unwrap();
            Ok(PROJECT_CONFIG.get().unwrap())
        }
    }
}

/// Deserializes the project's `config.toml` into the `PROJECT_CONFIG`'s `OnceLock`.
/// The process will be finished if either the `config.toml` cannot be opened, cannot be deserialized or the project's config is already loaded.
pub fn load_config() -> Result<project::Project> {
    match read(get_project_root()?.join("config.toml")) {
        Ok(file_buffer) => match toml::from_slice::<project::Project>(&file_buffer) {
            Ok(config) => Ok(config),
            Err(err) => Err(anyhow!(
                "Cannot deserialize the `config.toml` file.%{}",
                err.to_string()
            )),
        },
        Err(err) => Err(anyhow!(
            "Cannot open the `config.toml` file. Are you sure that you are in the project's folder?%{}",
            err.to_string()
        )),
    }
}
