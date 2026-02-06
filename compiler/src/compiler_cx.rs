// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

#[derive(Constructor, Getters, Debug)]
#[getset(get = "pub")]
pub struct CompilerCx {
    project: project::Project,
    workspace_root: PathBuf,
}

impl CompilerCx {
    pub fn acquire() -> &'static Self {
        COMPILER_CX
            .get()
            .ok_or(anyhow!("Compiler context should have been initialized."))
            .unwrap()
    }

    /// Sets the `COMPILER_CX`'s `OnceLock`.
    /// NOTE: If compiler's context is set this method will panic.
    pub fn set_cx(self) {
        if !COMPILER_CX.initialized() {
            COMPILER_CX.set(self).unwrap();
        } else {
            panic!("Compiler context has already been initialized.");
        }
    }

    /// Builds the compiler's context by loading the project
    /// from the workspace's root.
    pub async fn from_workspace() -> Result<Self> {
        let workspace_root = Self::get_workspace_root()?;

        match read(workspace_root.join("project.toml")) {
            Ok(file_buffer) => match toml::from_slice::<project::Project>(&file_buffer) {
                Ok(project) => Ok(Self::new(project, workspace_root)),
                Err(err) => Err(anyhow!(
                    "Cannot deserialize the `project.toml` file.%{}",
                    err.to_string()
                )),
            },
            Err(err) => Err(anyhow!(
                "Cannot open the `project.toml` file. Are you sure that you are in the project's folder?%{}",
                err.to_string()
            )),
        }
    }

    /// Tries to find the project's workspace root path.
    fn get_workspace_root() -> Result<PathBuf> {
        let mut current_dir = current_dir().unwrap();
        if current_dir.join("project.toml").exists() {
            return Ok(current_dir);
        } else {
            while current_dir.pop() {
                if current_dir.join("project.toml").exists() {
                    return Ok(current_dir);
                }
            }
        };
        Err(anyhow!(
            "The project's worspace root path cannot be determined."
        ))
    }
}
