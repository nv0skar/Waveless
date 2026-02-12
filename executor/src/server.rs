// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

#[instrument(skip_all)]
pub async fn serve(
    addr: Option<SocketAddr>,
    frontend: BoxCloneService<RouterRequest, Response<String>, Infallible>,
) -> Result<ResultContext> {
    let _build_lock = RuntimeCx::acquire().build();

    let listener = tokio::net::TcpListener::bind(
        addr.unwrap_or(
            _build_lock
                .read()
                .await
                .executor()
                .listening_addr()
                .ok_or(anyhow!("No server address was provided."))?,
        ),
    )
    .await
    .unwrap();

    info!(
        "Executing '{}' on {} at {}",
        _build_lock.read().await.config().name(),
        listener.local_addr().unwrap(),
        chrono::Local::now()
    );

    let governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(1000) // TODO: Make this a parameter in the 'project.toml'.
            .key_extractor(GlobalKeyExtractor) // TODO: Change this setting to allow IP-based rate limiting.
            .finish()
            .unwrap(),
    );

    // TODO: A POST request to an endpoint invalidates the caches of the GET endpoints with the same route.
    let cache = CacheLayer::builder(InMemoryBackend::new(4096))
        .ttl(Duration::from_secs(1)) // TODO: Make this a parameter in the 'project.toml'.
        .stale_while_revalidate(Duration::from_secs(1))
        .build();

    // Cleans up the governor key pool.
    let governor_limiter = governor_conf.limiter().to_owned();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        debug!(
            "Cleaning up rate's limiter storage (size: {})",
            governor_limiter.len()
        );
        governor_limiter.retain_recent();
    });

    // This would have worked ad-hoc without modifying the original crate if it has implemented `From<String>` for `GovernorError`...
    let governor = tower_governor::GovernorLayer::new(governor_conf).error_handler(|err| {
        Response::builder()
            .status(http::StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "application/json")
            .body(
                serde_json::to_string_pretty(&json!({
                    "error": err.to_compact_string()
                }))
                .unwrap(),
            )
            .unwrap()
    });

    let compression = CompressionLayer::new().compress_when(predicate::SizeAbove::new(2048));

    let endpoint_svc = ServiceBuilder::new()
        .layer(ExecuteWrapperLayer)
        .layer(RequestParamsExtractorLayer)
        .layer(AuthCaptureLayer)
        .layer(SessionWatchdogLayer)
        .service(ExecuteHandler);

    let router = services::RouterService::new(endpoint_svc, Some(frontend));

    let svc = ServiceBuilder::new()
        .layer(cache)
        .layer(compression)
        .layer(CorsLayer::permissive())
        .layer(TimeoutLayer::with_status_code(
            http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(governor) // Rate limiting does not apply for cached requests.
        .service(router);

    let svc = TowerToHyperService::new(svc);

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        let svc = svc.to_owned();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                error!(
                    "Internal error occurred while building a new connection handler: {:?}",
                    err
                );
            }
        });
    }
}
