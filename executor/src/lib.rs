// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod frontend_options;
pub mod request;
pub mod runtime_cx;
pub mod server;

pub use runtime_cx::*;

use waveless_commons::*;

use endpoint::*;
use waveless_commons::build::*;
use waveless_commons::execute::*;

use rustyrosetta::*;

use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::read;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use clap::Subcommand;
use compact_str::*;
use dashmap::DashMap;
use derive_more::Constructor;
use getset::*;
use http::StatusCode;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::Incoming,
    server::conn::{http1, http2},
    service::service_fn,
    *,
};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use matchit::*;
use serde_json::json;
use tokio::sync::{OnceCell, RwLock};
use tower::ServiceBuilder;
use tower_governor::{governor::*, key_extractor::*};
use tower_http::{compression::*, cors::*, timeout::*};
use tower_http_cache::prelude::*;
use tracing::*;

pub type EndpointRouter = DashMap<HttpMethod, Router<Endpoint>>;

pub static RUNTIME_CX: OnceCell<RuntimeCx> = OnceCell::const_new();
