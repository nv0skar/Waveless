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

        #[arg(help = "Listening address.")]
        addr: Option<SocketAddr>,

        #[arg(long = "tls_cert", help = "TLS cert path.")]
        tls_cert: Option<PathBuf>,

        #[arg(long = "tls_cert_key", help = "TLS cert key path.")]
        tls_cert_key: Option<PathBuf>,
    },
}
