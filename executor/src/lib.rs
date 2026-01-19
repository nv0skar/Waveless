// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod build_loader;
pub mod frontend_options;
pub mod server;

use waveless_binary::*;
use waveless_commons::*;
use waveless_config::*;
use waveless_schema::*;

use rustyrosetta::{codec::*, *};

use std::fs::{File, create_dir, read, read_dir, write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use clap::Subcommand;
use compact_str::*;
use derive_more::{Constructor, Display};
use serde::{Deserialize, Serialize};
use tracing::*;

pub static BUILD: OnceLock<binary::Build> = OnceLock::new();
