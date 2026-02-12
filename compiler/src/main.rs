// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

//!
//! The Waveless' frontend.
//!

use waveless_commons::{logger::*, runtime::handle_main, *};
use waveless_compiler::{build::*, compiler_cx::*, new::*};
use waveless_executor::{frontend_options::*, server::serve, *};

use build::*;
use databases::*;

use rustyrosetta::*;

use std::net::SocketAddr;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use compact_str::*;
use mimalloc::MiMalloc;
use nestify::nest;
use tower::{service_fn, util::BoxCloneService};
use tracing::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

nest! {
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

        /// Whether to skip endpoint discovery and only include user-defined endpoints (this overrides the `project.toml` file)
        #[arg(short = 'S', long = "skip_endpoint_discovery", default_value_t = false, help = "Whether to skip endpoint discovery and only include user-defined endpoints (this overrides the `project.toml` file)")]
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

fn main() -> Result<()> {
    handle_main(try_main)
}

async fn try_main() -> Result<ResultContext> {
    let cli = Frontend::parse();

    // Setup logging
    subscribe_logger(cli.debug)?;

    // Handle frontend subcommands
    match cli.subcommand {
        Some(Subcommands::New { name }) => new_project(name),
        Some(Subcommands::Run { addr }) => {
            CompilerCx::set_cx(CompilerCx::from_workspace().await?);

            let build = build::<ExecutorBuild>().await?.left().unwrap();

            RuntimeCx::set_cx(RuntimeCx::from_build(build).await?);

            let _build_lock = RuntimeCx::acquire().build();

            if *_build_lock
                .read()
                .await
                .executor()
                .check_databases_cheksums()
            {
                warn!("Skipping databases' schema checksum verification.");
            }

            DatabasesConnections::load(_build_lock.read().await.config().databases().to_owned())
                .await?;

            serve(
                addr,
                BoxCloneService::new(service_fn(|_| async {
                    todo!("Frontend not implemented yet.")
                })),
            )
            .await?;

            return Ok("".to_compact_string());
        }
        Some(Subcommands::Build) => {
            CompilerCx::set_cx(CompilerCx::from_workspace().await?);
            let buff = build::<Bytes>().await?.right().unwrap();
            binary_file_from_buff(buff)
        }
        Some(Subcommands::Bootstrap) => todo!(),
        Some(Subcommands::Executor(executor_options)) => match executor_options {
            ExecutorFrontendOptions::Run { path, addr } => {
                RuntimeCx::set_cx(RuntimeCx::from_path(path).await?);

                let _build_lock = RuntimeCx::acquire().build();

                if *_build_lock
                    .read()
                    .await
                    .executor()
                    .check_databases_cheksums()
                {
                    check_checksums_in_build(&(*_build_lock.read().await)).await?;
                }

                DatabasesConnections::load(
                    _build_lock.read().await.config().databases().to_owned(),
                )
                .await?;

                serve(
                    addr,
                    BoxCloneService::new(service_fn(|_| async {
                        todo!("Frontend not implemented yet.")
                    })),
                )
                .await?;

                Ok("".to_compact_string())
            }
        },
        None => Err(anyhow!("No subcommdand provided!")),
    }
}
