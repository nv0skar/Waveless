// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod handshake_cx;

use handshake_cx::*;

use crate::*;

use databases::*;

/// Generic methods trait to handle requests to the endpoints.
#[typetag::serde]
#[async_trait]
pub trait AnySocketExecute: Any + BoxedAny + DynClone + Send + Sync + Debug {
    /// Executes a query using the given executor and database connection.
    async fn execute(
        &self,
        handshake_cx: HandshakeCx,
        websocket: HyperWebsocket,
        db_conn: Arc<dyn AnyDatabaseConnection>,
    ) -> Result<(), Infallible>;
}
