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
            .ok_or(eyre!("Compiler context should have been initialized."))
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
        let workspace_root = get_workspace_root("project.toml")?;

        match read(workspace_root.join("project.toml")) {
            Ok(file_buffer) => match toml::from_slice::<project::Project>(&file_buffer) {
                Ok(project) => Ok(Self::new(project, workspace_root)),
                Err(err) => Err(err).wrap_err("Cannot deserialize the `project.toml` file."),
            },
            Err(err) => Err(err)
                .wrap_err("Cannot open the `project.toml` file.")
                .suggestion("Are you sure that you are in the project's folder?"),
        }
    }
}
