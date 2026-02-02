// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use waveless_binary::*;
use waveless_commons::{logger::*, runtime::handle_main, *};
use waveless_compiler::{build::*, new::*};
use waveless_databases::*;
use waveless_executor::{
    build_loader::load_build, frontend_options::*, router_loader::*, server::serve, *,
};

use rustyrosetta::*;

use std::net::SocketAddr;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use compact_str::*;
use iocraft::prelude::*;
use mimalloc::MiMalloc;
use nestify::nest;
use tracing::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

nest! {
    ///
    /// The Waveless' frontend.
    ///
    #[derive(Parser)]
    #[command(
        name = "waveless",
        version,
        about = "The Waveless' frontend.",
        long_about = "Analyze and build the project in the current directory and generate a Waveless' binary.",
        propagate_version = true,
        subcommand_required = true,
        arg_required_else_help = true,
    )]
    struct Frontend {
        /// Whether to enable debug mode in the compiler.
        #[arg(short = 'D', long = "debug", default_value_t = false)]
        debug: bool,

        /// Whether to show all included endpoints in the build file.
        #[arg(short = 'd', long = "display_endpoints", default_value_t = true, help = "Whether to show all included endpoints in the build file.")]
        display_endpoints_on_build: bool,

        /// Whether to skip endpoint discovery and only include user-defined endpoints (this overrides the `config.toml` file)
        #[arg(short = 'S', long = "skip_endpoint_discovery", default_value_t = false, help = "Whether to skip endpoint discovery and only include user-defined endpoints (this overrides the `config.toml` file)")]
        skip_endpoint_discovery: bool,

        /// All cli subcommands
        #[command(subcommand)]
        subcommand: Option<
            #[derive(Subcommand)]
            enum Subcommands {
                /// Creates a new Waveless' project.
                #[command(about = "Creates a new Waveless' project.")]
                New {
                    #[arg(help = "Project's name")]
                    name: CompactString,
                },

                /// Builds and launches the server executor using the outputted binary.
                #[command(about = "Builds and launches the server executor using the outputted binary.")]
                Run {
                    #[arg(help = "Listening address.")]
                    addr: Option<SocketAddr>,
                },

                /// Builds the current project.
                #[command(about = "Builds the current project.")]
                Build,

                /// Bootstraps the database, running all the scripts under the specified `bootstrap_scripts_dir` folder.
                #[command(about = "Bootstraps the database, running all the scripts under the specified `bootstrap_scripts_dir` folder.")]
                Bootstrap,

                /// The Waveless' executor.
                #[command(about = "The Waveless' executor.", subcommand)]
                Executor(ExecutorFrontendOptions)
            }
        >
    }
}

fn main() {
    handle_main(try_main);
}

async fn try_main() -> Result<ResultContext> {
    let cli = Frontend::parse();

    // Setup logging
    subscribe_logger(cli.debug)?;

    // Handle frontend subcommands
    match cli.subcommand {
        Some(Subcommands::New { name }) => new_project(name),
        Some(Subcommands::Run { addr }) => {
            let build = build::<binary::Build>().await?.left().unwrap();

            BUILD
                .set(build.to_owned())
                .map_err(|_| anyhow!("Cannot load build into global."))?;

            ROUTER
                .set(load_router()?)
                .map_err(|_| anyhow!("Cannot load router into global."))?;

            if *build.server_settings().check_databases_cheksums() {
                warn!("Skipping databases' schema checksum verification.");
            }

            DatabasesConnections::load(build.general().databases().to_owned()).await?;

            serve(addr).await
        }
        Some(Subcommands::Build) => {
            let buff = build::<Bytes>().await?.right().unwrap();
            binary_file_from_buff(buff)
        }
        Some(Subcommands::Bootstrap) => todo!(),
        Some(Subcommands::Executor(executor_options)) => match executor_options {
            ExecutorFrontendOptions::Run { path, addr } => {
                BUILD
                    .set(load_build(path)?)
                    .map_err(|_| anyhow!("Cannot load build into global."))?;

                ROUTER
                    .set(load_router()?)
                    .map_err(|_| anyhow!("Cannot load router into global."))?;

                let build = build_loader::build()?.to_owned();

                if *build.server_settings().check_databases_cheksums() {
                    check_checksums_in_build(build.to_owned()).await?;
                }

                DatabasesConnections::load(build.general().databases().to_owned()).await?;

                serve(addr).await
            }
        },
        None => Err(anyhow!("No subcommand provided!")),
    }
}
