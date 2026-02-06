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

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::HashMap;
use std::env::var;
use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::sync::Arc;

use rustyrosetta::{codec::*, *};

use anyhow::{Context, Result, anyhow, bail};
use arrayvec::ArrayVec;
use async_trait::*;
use compact_str::*;
use derive_builder::*;
use derive_more::{Constructor, Display};
use dyn_clone::*;
use getset::*;
use iocraft::prelude::*;
use serde::{Deserialize, Serialize};
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
