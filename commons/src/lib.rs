// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod logger;
pub mod output;
pub mod serialize;

use std::cell::Cell;

use rustyrosetta::*;

use anyhow::{Context, Result};
use compact_str::*;
use iocraft::prelude::*;
use tracing::*;

pub type ResultContext = CompactString; // TODO: Replace this with custom error types.

thread_local! {
    pub static BINARY_MODE: Cell<bool> = Cell::new(false); // This will likely be fixed in the future. https://github.com/serde-rs/serde/issues/1732
}
