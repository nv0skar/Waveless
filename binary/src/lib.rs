// Waveless
// Copyright (C) 2025 Oscar Alvarez Gonzalez

pub mod binary;

use waveless_config::*;
use waveless_schema::*;

use rustyrosetta::*;

use anyhow::*;
use compact_str::*;
use derive_more::{Constructor, Display};
use getset::*;
use serde::{Deserialize, Serialize};
