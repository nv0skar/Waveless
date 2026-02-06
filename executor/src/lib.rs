// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod build_loader;
pub mod frontend_options;
pub mod request;
pub mod router_loader;
pub mod server;

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
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use clap::Subcommand;
use compact_str::*;
use dashmap::DashMap;
use derive_more::Constructor;
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
use sea_orm::{FromQueryResult, QueryResult}; // Switched from sqlx, as sqlx doesn't support conversion into JSON for arbitrary schemas.
use serde_json::json;
use thiserror::Error;
use tokio::sync::OnceCell;
use tower::ServiceBuilder;
use tower_governor::{governor::*, key_extractor::*};
use tower_http::{compression::*, cors::*, timeout::*};
use tower_http_cache::prelude::*;
use tracing::*;

pub type EndpointRouter = DashMap<HttpMethod, Router<Endpoint>>;

pub static BUILD: OnceLock<Build> = OnceLock::new();

pub static ROUTER: OnceCell<EndpointRouter> = OnceCell::const_new();
