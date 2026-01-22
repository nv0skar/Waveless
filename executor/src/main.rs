// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use waveless_commons::{logger::*, output::handle_main, *};
use waveless_databases::*;
use waveless_executor::{build_loader::*, frontend_options::*, router_loader::*, server::*, *};

use anyhow::{Context, Result, anyhow, bail};
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

            for build_checksum in build()?.databases_checksums() {
                let db_config = build()?
                    .general()
                    .databases()
                    .iter()
                    .find(|db_config| db_config.id() == build_checksum.database_id())
                    .ok_or(anyhow!("No database with the matching criteria was found."))?;

                let schema = waveless_databases::schema::AnySchema::load_schema(db_config).await?;

                let current_checksum = schema
                    .checksum(build_checksum.database_id().to_owned())
                    .await?;

                if current_checksum != *build_checksum {
                    bail!(
                        "The database schema has changed since the last build! Build the project again using the current schema."
                    )
                }
            }

            DatabasesConnections::load(build()?.general().databases().to_owned()).await?;

            serve(addr).await
        }
        None => Err(anyhow!("No subcommand provided!")),
    }
}
