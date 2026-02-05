// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// Retrieves the project's build from the `BUILD`'s `OnceLock` or panics if it is not present.
pub fn build() -> Result<&'static Build> {
    match BUILD.get() {
        Some(build) => Ok(build),
        None => {
            panic!("The `BUILD` global is not set.")
        }
    }
}

/// Deserializes the project's build into the `BUILD`'s `OnceLock`.
pub fn load_build(path: PathBuf) -> Result<Build> {
    match read(path.to_owned()) {
        Ok(file_buffer) => match Build::decode_binary(&CheapVec::from_vec(file_buffer)) {
            Ok(build) => Ok(build),
            Err(err) => Err(anyhow!(
                "Cannot deserialize the binary '{}'.%{}",
                path.display(),
                err.to_string()
            )),
        },
        Err(err) => Err(anyhow!(
            "Cannot open '{}'. Are you sure that you have the file's permissions?%{}",
            path.display(),
            err.to_string()
        )),
    }
}
