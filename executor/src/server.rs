// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

pub async fn serve(addr: Option<SocketAddr>) -> Result<ResultContext> {
    let build = build_loader::build()?;

    let listener = tokio::net::TcpListener::bind(
        addr.unwrap_or(
            build
                .server_settings()
                .listening_addr()
                .ok_or(anyhow!("No server address was provided."))?,
        ),
    )
    .await
    .unwrap();

    info!(
        "Executing '{}' on {} at {}",
        build.general().name(),
        listener.local_addr().unwrap(),
        chrono::Local::now()
    );

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(request_handler::try_handle_endpoint))
                .await
            {
                error!("Internal error occurred in request handler: {:?}", err);
            }
        });
    }
}
