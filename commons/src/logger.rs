// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// Setups logging
pub fn subscribe_logger(debug: bool) -> Result<()> {
    // Setup logging
    let stdout_subscriber = tracing_subscriber::FmtSubscriber::builder()
        .without_time()
        .with_target(false)
        .compact()
        .with_max_level(if debug { Level::DEBUG } else { Level::INFO })
        .finish();

    tracing::subscriber::set_global_default(stdout_subscriber)
        .context("Setting tracing subscriber failed")?;

    Ok(())
}
