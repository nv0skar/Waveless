// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod binary;

use waveless_config::*;
use waveless_schema::*;

use rustyrosetta::{codec::*, *};

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use derive_more::Constructor;
use getset::*;
use serde::{Deserialize, Serialize};

pub const BINARY_MAGIC: &'static [u8] = b"waveless_binary";
