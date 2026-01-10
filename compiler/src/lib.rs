// Waveless
// Copyright (C) 2025 Oscar Alvarez Gonzalez

pub mod bootstrap;
pub mod build;
pub mod new;

use waveless_config::*;
use waveless_schema::*;

use rustyrosetta::*;

use std::env::current_dir;
use std::fs::{File, create_dir};
use std::io::Write;
use std::path::Path;
use std::process::exit;

use anyhow::*;
use compact_str::*;
use derive_more::{Constructor, Display};
use owo_colors::*;
use serde::{Deserialize, Serialize};
use tracing::*;
