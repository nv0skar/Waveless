// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod auth;
pub mod build;
pub mod databases;
pub mod endpoint;
pub mod entry;
pub mod execute;
pub mod logger;
pub mod project;
pub mod runtime;
pub mod schema;

mod serialize_utils;

pub use serialize_utils::*;

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::HashMap;
use std::env::{current_dir, var};
use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rustyrosetta::{codec::*, *};

use anyhow::{Context, Result, anyhow, bail};
use arrayvec::ArrayVec;
use async_trait::*;
use chrono::{NaiveDateTime, Utc};
use compact_str::*;
use derive_builder::*;
use derive_more::{Constructor, Display};
use dyn_clone::*;
use getset::*;
use http::StatusCode;
use iocraft::prelude::*;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::*;
use tokio::{runtime::Builder, sync::OnceCell};
use tracing::*;

pub type ResultContext = CompactString; // TODO: Replace this with custom error types → `thiserror`

pub type DatabaseId = CompactString;
pub type DataSchemaDiscoveryMethodId = CompactString;
pub type ExternalDriverId = CompactString;

pub type UserId = usize;

/// The binary's prefix.
pub const BINARY_MAGIC: &'static [u8] = b"waveless_binary";

/// The maximum number of databases the user's application can connect to.
pub const DATABASE_LIMIT: usize = 9;

pub static DATABASES_CONNS: OnceCell<databases::DatabasesConnections> = OnceCell::const_new();

thread_local! {
    pub static BINARY_MODE: Cell<bool> = const { Cell::new(false) }; // This will likely be fixed in the future. https://github.com/serde-rs/serde/issues/1732
}

pub trait BoxedAny {
    fn as_boxed_any(&'static self) -> Box<dyn Any>;
    fn as_arc_any(&'static self) -> Arc<dyn Any + Send + Sync + 'static>;
    fn into_boxed_any(self: Box<Self>) -> Box<dyn Any>;
    fn into_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static>;
    fn inner_type_id(&self) -> TypeId;
}

/// TODO: this should be a derive macro.
#[macro_export]
macro_rules! boxed_any {
    ($type:ty) => {
        impl BoxedAny for $type {
            fn as_boxed_any(&'static self) -> Box<dyn Any> {
                Box::new(self)
            }

            fn as_arc_any(&'static self) -> Arc<dyn Any + Send + Sync + 'static> {
                Arc::new(self)
            }

            fn into_boxed_any(self: Box<Self>) -> Box<dyn Any> {
                self
            }

            fn into_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static> {
                self
            }

            fn inner_type_id(&self) -> TypeId {
                TypeId::of::<$type>()
            }
        }
    };
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Request error.")]
    Expected(StatusCode, CompactString),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Tries to find the project's workspace root path.
pub fn get_workspace_root(project_file: &str) -> Result<PathBuf> {
    let mut current_dir = current_dir().unwrap();
    if current_dir.join(project_file).exists() {
        return Ok(current_dir);
    } else {
        while current_dir.pop() {
            if current_dir.join(project_file).exists() {
                return Ok(current_dir);
            }
        }
    };
    Err(anyhow!(
        "The project's worspace root path cannot be determined."
    ))
}
