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
    debug!("Listening on {}", listener.local_addr().unwrap());
    // axum::serve(listener, app(client)).await;
    Ok("".to_compact_string())
}

// fn app() -> Router {
//     let collection: Collection<Member> = client.database("axum-mongo").collection("members");

//     Router::new()
//         .route("/create", post(create_member))
//         .route("/read/{id}", get(read_member))
//         .route("/update", put(update_member))
//         .route("/delete/{id}", delete(delete_member))
//         .layer(TraceLayer::new_for_http())
//         .with_state(collection)
// }
