// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod build;
pub mod databases;
pub mod endpoint;
pub mod entry;
pub mod logger;
pub mod project;
pub mod runtime;
pub mod schema;

mod serialize_utils;

pub use serialize_utils::*;

use std::any::Any;
use std::cell::Cell;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::net::SocketAddr;

use rustyrosetta::{codec::*, *};

use anyhow::{Context, Result, anyhow, bail};
use arrayvec::ArrayVec;
use compact_str::*;
use derive_builder::*;
use derive_more::{Constructor, Display};
use getset::*;
use iocraft::prelude::*;
use rclite::Arc;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use smallbox::{space::S64, *};
use tokio::{runtime::Builder, sync::OnceCell};
use tracing::*;

pub type ResultContext = CompactString; // TODO: Replace this with custom error types → `thiserror`

pub type DatabaseId = CompactString;
pub type DataSchemaDiscoveryMethodId = CompactString;
pub type ExternalDriverId = CompactString;

/// The binary's prefix.
pub const BINARY_MAGIC: &'static [u8] = b"waveless_binary";

/// The maximum number of databases the user's application can connect to.
pub const DATABASE_LIMIT: usize = 9;

thread_local! {
    pub static BINARY_MODE: Cell<bool> = const { Cell::new(false) }; // This will likely be fixed in the future. https://github.com/serde-rs/serde/issues/1732
}
