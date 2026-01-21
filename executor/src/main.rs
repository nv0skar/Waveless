// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use waveless_commons::{logger::*, output::handle_main, *};
use waveless_executor::{build_loader::*, frontend_options::*, router_loader::*, server::*, *};

use anyhow::{Context, Result, anyhow};
use clap::Parser;

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

#[tokio::main]
async fn main() {
    handle_main(try_main).await
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

            databases::DatabasesConnections::load().await?;

            serve(addr).await
        }
        None => Err(anyhow!("No subcommand provided!")),
    }
}
