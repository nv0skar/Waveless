// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// Compiler's frontend's options.
#[derive(Subcommand)]
pub enum ExecutorFrontendOptions {
    /// Launches the server runtime with the specified Waveless' binary.
    #[command(about = "Launches the server runtime with the specified Waveless' binary.")]
    Run {
        #[arg(help = "Binary path.")]
        path: PathBuf,
    },
}
