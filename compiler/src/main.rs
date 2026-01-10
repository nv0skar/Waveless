// Waveless
// Copyright (C) 2025 Oscar Alvarez Gonzalez

use waveless_compiler::new::*;

use rustyrosetta::*;

use anyhow::*;
use clap::{Parser, Subcommand};
use compact_str::*;
use nestify::nest;
use tracing::*;

nest! {
    ///
    /// The Waveless' compiler frontend/cli.
    /// A note to error handling: there will be to ways the program may exit due to an error:
    /// 1. Unexpected errors: these are propagated back to the `main` entry.
    /// 2. Expected or user errors: these are errors that occur commonly, thus these will be handled nicely with verbose error messages
    /// and proper program termination
    ///
    #[derive(Parser)]
    #[command(
        name = "waveless",
        version,
        about = "The Waveless' compiler.",
        long_about = "Analyze and build the project in the current directory and generate a Waveless' binary.",
        propagate_version = true,
        subcommand_required = true,
        arg_required_else_help = true,
    )]
    struct CompilerCli {
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
                Run,

                /// Builds the current project.
                #[command(about = "Builds the current project.")]
                Build,

                /// Bootstraps the database, running all the scripts under the specified `bootstrap_scripts_dir` folder.
                #[command(about = "Bootstraps the database, running all the scripts under the specified `bootstrap_scripts_dir` folder.")]
                Bootsrap,
            }
        >
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CompilerCli::parse();

    // Setup logging
    let stdout_subscriber = tracing_subscriber::FmtSubscriber::builder()
        .without_time()
        .with_target(false)
        .compact()
        .with_max_level(if cli.debug { Level::DEBUG } else { Level::INFO })
        .finish();

    tracing::subscriber::set_global_default(stdout_subscriber)
        .context("Setting tracing subscriber failed")?;

    // Handle cli subcommands
    if let Some(subcommand) = cli.subcommand {
        match subcommand {
            Subcommands::New { name } => new_project(name)?,
            Subcommands::Run => todo!(),
            Subcommands::Build => todo!(),
            Subcommands::Bootsrap => todo!(),
        };
    };

    Ok(())
}
