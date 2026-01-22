// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod build_loader;
pub mod execute;
pub mod frontend_options;
pub mod request;
pub mod router_loader;
pub mod server;

use request::ConnHandlerError;

use waveless_binary::*;
use waveless_commons::*;
use waveless_config::*;
use waveless_databases::*;
use waveless_schema::*;

use rustyrosetta::{codec::*, *};

use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::{File, create_dir, read, read_dir, write};
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::sync::{LazyLock, OnceLock};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use arrayvec::ArrayVec;
use clap::Subcommand;
use compact_str::*;
use dashmap::DashMap;
use derive_more::{Constructor, Display};
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
use rclite::Arc;
use sea_orm::{
    DbBackend, FromQueryResult, SqlxMySqlPoolConnection, Statement, entity::*, prelude::*,
}; // Switched from sqlx, as sqlx doesn't support conversion into JSON for arbitrary schemas.
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{mysql::*, pool::*};
use thiserror::Error;
use tokio::{net::TcpListener, sync::OnceCell};
use tower::{Service, ServiceBuilder, limit::RateLimitLayer};
use tower_governor::{governor::*, key_extractor::*, *};
use tower_http::{compression::*, cors::*, timeout::*};
use tower_http_cache::prelude::*;
use tracing::*;

pub type EndpointRouter = DashMap<endpoint::HttpMethod, Router<endpoint::Endpoint>>;

pub static BUILD: OnceLock<binary::Build> = OnceLock::new();

pub static ROUTER: OnceCell<EndpointRouter> = OnceCell::const_new();
