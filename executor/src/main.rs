// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

//!
//! The Waveless' executor frontend.
//!

use waveless_commons::{databases::*, logger::*, runtime::handle_main, *};
use waveless_executor::{frontend_options::*, router_loader::*, runtime_build::*, server::*, *};

use std::sync::Arc;

use anyhow::{Result, anyhow};
use clap::Parser;
use compact_str::*;
use mimalloc::MiMalloc;
use tokio::sync::RwLock;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Parser)]
#[command(
    name = "waveless_executor",
    version,
    about = "The Waveless' executor.",
    long_about = "Waveless' binary runtime.",
    propagate_version = true,
    subcommand_required = true,
    arg_required_else_help = true
)]
struct ExecutorFrontend {
    /// Whether to enable debug mode in the executor.
    #[arg(short = 'D', long = "debug", default_value_t = false)]
    debug: bool,

    /// All cli subcommands
    #[command(subcommand)]
    subcommand: Option<ExecutorFrontendOptions>,
}

fn main() -> Result<()> {
    handle_main(try_main)
}

async fn try_main() -> Result<ResultContext> {
    let cli = ExecutorFrontend::parse();

    // Setup logging
    subscribe_logger(cli.debug)?;

    // Handle frontend subcommands
    match cli.subcommand {
        Some(ExecutorFrontendOptions::Run { path, addr }) => {
            RUNTIME_BUILD
                .set(Arc::new(RwLock::new(load_build_from_file(path)?)))
                .map_err(|_| anyhow!("Cannot load build into global."))?;

            ROUTER
                .set(load_router().await?)
                .map_err(|_| anyhow!("Cannot load router into global."))?;

            let _build_lock = build().await?;

            if *_build_lock
                .read()
                .await
                .server_settings()
                .check_databases_cheksums()
            {
                check_checksums_in_build(&(*_build_lock.read().await)).await?;
            }

            DatabasesConnections::load(_build_lock.read().await.config().databases().to_owned())
                .await?;

            serve(addr).await
        }
        None => Err(anyhow!("No subcommand provided!")),
    }
}
