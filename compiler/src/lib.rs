// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod bootstrap;
pub mod build;
pub mod compiler_cx;
pub mod discovery;
pub mod new;

pub use compiler_cx::*;

use waveless_commons::*;

use endpoint::*;
use execute::mysql::*;
use waveless_commons::build::*;

use rustyrosetta::*;

use std::any::TypeId;
use std::env::current_dir;
use std::fs::{File, create_dir, read, read_dir, write};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use compact_str::*;
use derive_more::Constructor;
use either::*;
use getset::*;
use owo_colors::*;
use tokio::sync::OnceCell;
use tracing::*;

pub static COMPILER_CX: OnceCell<CompilerCx> = OnceCell::const_new();

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
