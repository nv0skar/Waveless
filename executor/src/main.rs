// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use waveless_commons::{logger::*, runtime::handle_main, *};
use waveless_databases::*;
use waveless_executor::{build_loader::*, frontend_options::*, router_loader::*, server::*, *};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use tracing::*;

///
/// The Waveless' executor frontend.
///
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

fn main() {
    handle_main(try_main)
}

async fn try_main() -> Result<ResultContext> {
    let cli = ExecutorFrontend::parse();

    // Setup logging
    subscribe_logger(cli.debug)?;

    // Handle frontend subcommands
    match cli.subcommand {
        Some(ExecutorFrontendOptions::Run { path, addr }) => {
            BUILD
                .set(load_build(path)?)
                .map_err(|_| anyhow!("Cannot load build into global."))?;

            ROUTER
                .set(load_router()?)
                .map_err(|_| anyhow!("Cannot load router into global."))?;

            if *build()?.server_settings().check_databases_cheksums() {
                check_checksums_in_build(build()?.to_owned()).await?;
            }

            DatabasesConnections::load(build()?.general().databases().to_owned()).await?;

            serve(addr).await
        }
        None => Err(anyhow!("No subcommand provided!")),
    }
}
