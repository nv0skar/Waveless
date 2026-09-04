// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod bootstrap;
pub mod compiler_cx;
pub mod generator;
pub mod new;
pub mod workspace;

pub use compiler_cx::*;

use waveless_commons::{endpoint::*, object::*, *};

use rustyrosetta::*;

use std::any::TypeId;
use std::env::current_dir;
use std::fs::{File, create_dir, read, read_dir, write};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use color_eyre::Section;
use compact_str::*;
use derive_more::Constructor;
use either::*;
use eyre::{Context, Result, eyre};
use getset::*;
use tokio::sync::OnceCell;
use tracing::*;

pub static COMPILER_CX: OnceCell<CompilerCx> = OnceCell::const_new();
