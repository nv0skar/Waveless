// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod bootstrap;
pub mod build;
pub mod config_loader;
pub mod discovery;
pub mod new;

use waveless_commons::*;

use endpoint::*;
use schema::*;
use waveless_commons::build::*;

use rustyrosetta::*;

use std::any::{Any, TypeId};
use std::env::current_dir;
use std::fs::{File, create_dir, read, read_dir, write};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use compact_str::*;
use derive_more::Constructor;
use either::*;
use owo_colors::*;
use tracing::*;

pub static PROJECT_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub static PROJECT_CONFIG: OnceLock<project::Project> = OnceLock::new();

/// Get's the project's root folder's path.
pub fn get_project_root() -> Result<PathBuf> {
    match PROJECT_ROOT.get() {
        Some(path) => Ok(path.to_owned()),
        None => {
            let mut current_dir = current_dir().unwrap();
            if current_dir.join("config.toml").exists() {
                PROJECT_ROOT.set(current_dir.to_owned()).unwrap();
                return Ok(current_dir);
            } else {
                while current_dir.pop() {
                    if current_dir.join("config.toml").exists() {
                        PROJECT_ROOT.set(current_dir.to_owned()).unwrap();
                        return Ok(current_dir);
                    }
                }
            };
            Err(anyhow!("The project's path cannot be determined."))
        }
    }
}

pub fn expected_error(reason: String, hint: Option<&'static str>, error: String) {
    println!(
        "{} {}",
        "ERROR:".bright_red().bold(),
        format!("{} {:?}", reason, error).bright_white()
    );
    if let Some(hint) = hint {
        println!("❓ {}", hint.bright_white());
    }
    exit(1);
}
