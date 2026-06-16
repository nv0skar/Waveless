// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use tracing_subscriber::{
    Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Setups logging
pub fn subscribe_logging(debug: bool) -> Result<()> {
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .without_time()
        .with_target(false)
        .with_filter(if debug {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        });

    let registry = tracing_subscriber::registry().with(stdout_layer);

    #[cfg(debug_assertions)]
    let registry = registry.with(console_subscriber::spawn());

    registry.try_init().context("Tracing subscriber failed.")?;

    Ok(())
}
