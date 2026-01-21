// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod build_loader;
pub mod databases;
pub mod execute_handler;
pub mod frontend_options;
pub mod request_handler;
pub mod router_loader;
pub mod server;

use databases::*;
use request_handler::ConnHandlerError;

use waveless_binary::*;
use waveless_commons::*;
use waveless_config::*;
use waveless_schema::*;

use rustyrosetta::{codec::*, *};

use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::{File, create_dir, read, read_dir, write};
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, OnceLock};

use anyhow::{Context, Result, anyhow, bail};
use arrayvec::ArrayVec;
use clap::Subcommand;
use compact_str::*;
use dashmap::DashMap;
use derive_more::{Constructor, Display};
use http_body_util::Full;
use hyper::{
    body::{Body, Bytes},
    server::conn::{http1, http2},
    service::service_fn,
    *,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use matchit::*;
use owo_colors::*;
use sea_orm::{
    DbBackend, FromQueryResult, SqlxMySqlPoolConnection, Statement, entity::*, prelude::*,
    query::JsonValue,
}; // Switched from sqlx, as sqlx doesn't support conversion into JSON for arbitrary schemas.
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{mysql::*, pool::*};
use thiserror::Error;
use tokio::{net::TcpListener, sync::OnceCell};
use tracing::*;

pub type EndpointRouter = DashMap<endpoint::HttpMethod, Router<endpoint::Endpoint>>;

pub static BUILD: OnceLock<binary::Build> = OnceLock::new();

pub static DATABASES_CONNS: OnceCell<DatabasesConnections> = OnceCell::const_new();

pub static ROUTER: OnceCell<EndpointRouter> = OnceCell::const_new();
