// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use waveless_commons::logger::*;
use waveless_executor::frontend_options::*;

use anyhow::Result;
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
pub async fn main() -> Result<()> {
    let cli = ExecutorFrontend::parse();

    // Setup logging
    subscribe_logger(cli.debug)?;

    Ok(())
}
