// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use waveless_commons::logger::*;
use waveless_compiler::{build::*, new::*, *};
use waveless_executor::frontend_options::*;

use rustyrosetta::*;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use compact_str::*;
use iocraft::prelude::*;
use nestify::nest;
use tracing::*;

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
                Run,

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

#[tokio::main]
async fn main() {
    match try_main() {
        Ok(res) => {
            element! {
                View(
                    padding_left: 1,
                    padding_right: 1,
                    border_style: BorderStyle::Round,
                    border_color: iocraft::Color::Green,
                ) {
                    MixedText(align: TextAlign::Left, contents: vec![
                        MixedTextContent::new("✅ "),
                        MixedTextContent::new("SUCCESS: ").color(iocraft::Color::Green).weight(Weight::Bold),
                        MixedTextContent::new(res).color(iocraft::Color::White),
                    ])
                }
            }
            .print();
        }
        Err(err) => {
            let err = err.to_string();
            let (res, cx) = err.split_once("%").unwrap_or((err.as_str(), ""));
            element! {
                View(
                    padding_left: 1,
                    padding_right: 1,
                    border_style: BorderStyle::Round,
                    border_color: iocraft::Color::Red,
                ) {
                    MixedText(align: TextAlign::Left, contents: vec![
                        MixedTextContent::new("🔴 "),
                        MixedTextContent::new("ERROR: ").color(iocraft::Color::Red).weight(Weight::Bold),
                        MixedTextContent::new(res).color(iocraft::Color::White),
                        MixedTextContent::new(format!("\n{}", cx)).color(iocraft::Color::Blue),
                    ])
                }
            }
            .print();
        }
    }
}

fn try_main() -> Result<ResultContext> {
    let cli = Frontend::parse();

    // Setup logging
    subscribe_logger(cli.debug)?;

    // Handle frontend subcommands
    match cli.subcommand {
        Some(Subcommands::New { name }) => new_project(name),
        Some(Subcommands::Run) => todo!(),
        Some(Subcommands::Build) => build(),
        Some(Subcommands::Bootstrap) => todo!(),
        _ => todo!(),
    }
}
