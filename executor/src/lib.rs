// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod conn_executor;
pub mod frontend_options;
pub mod internal_endpoints;
pub mod runtime_cx;
pub mod server;
pub mod services;

pub use conn_executor::*;
pub use internal_endpoints::*;
pub use runtime_cx::*;
pub use services::*;

use waveless_commons::*;

use waveless_commons::build::*;
use waveless_commons::endpoint::*;
use waveless_commons::http_execute::{request_cx::*, *};
use waveless_commons::socket_execute::{handshake_cx::*, *};

use rustyrosetta::*;

use std::cell::LazyCell;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs::read;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

use anyhow::{Result, anyhow};
use clap::Subcommand;
use compact_str::*;
use dashmap::DashMap;
use derive_more::Constructor;
use futures::{SinkExt, StreamExt, future::BoxFuture};
use getset::*;
use http::{HeaderName, HeaderValue, StatusCode};
use http_body_util::BodyExt;
use hyper::{body::Incoming, *};
use hyper_tungstenite::tungstenite;
use hyper_util::{
    rt::TokioIo, server::conn::auto::Builder as AutoHttpBuilder, service::TowerToHyperService,
};
use matchit::*;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use serde_json::json;
use tokio::{
    fs::try_exists,
    sync::{OnceCell, RwLock},
};
use tokio_rustls::*;
use tower::{Layer, Service, ServiceBuilder, util::BoxCloneService};
use tower_governor::{governor::*, key_extractor::*};
use tower_http::{compression::*, cors::*, timeout::*};
use tower_http_cache::prelude::*;
use tracing::*;
use tungstenite::Message;

pub type EndpointRouter = DashMap<HttpMethod, Router<Endpoint>>;

pub static RUNTIME_CX: OnceCell<RuntimeCx> = OnceCell::const_new();
