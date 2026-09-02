// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use tracing_subscriber::{
    Layer, Registry, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Setups logging
pub fn subscribe_logging(
    debug: bool,
    ext_layers: CheapVec<Box<dyn Layer<Registry> + Send + Sync + 'static>>,
) -> Result<()> {
    let mut layers = Vec::from_iter(ext_layers);

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .without_time()
        .with_target(true)
        .with_filter(if debug {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        })
        .boxed();

    layers.push(stdout_layer);

    #[cfg(all(feature = "debug", not(target_arch = "wasm32")))]
    layers.push(console_subscriber::spawn().boxed());

    let registry = tracing_subscriber::registry().with(layers);

    registry.try_init().context("Tracing subscriber failed.")?;

    Ok(())
}
