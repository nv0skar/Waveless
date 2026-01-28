// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod binary;

use waveless_commons::*;
use waveless_config::*;
use waveless_endpoint::*;

use rustyrosetta::{codec::*, *};

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use clap::Subcommand;
use compact_str::*;
use derive_more::Constructor;
use getset::*;
use serde::{Deserialize, Serialize};
use tracing::*;

pub const BINARY_MAGIC: &'static [u8] = b"waveless_binary";
