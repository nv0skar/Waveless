// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod frontend_options;

use waveless_config::*;
use waveless_schema::*;

use rustyrosetta::{codec::*, *};

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use compact_str::*;
use derive_more::{Constructor, Display};
use getset::*;
use serde::{Deserialize, Serialize};
