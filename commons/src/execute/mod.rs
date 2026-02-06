// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod mysql;

use crate::*;

use databases::*;
use endpoint::*;

/// Generic methods trait to handle requests to the endpoints.
#[typetag::serde]
#[async_trait]
pub trait AnyExecute: Any + BoxedAny + DynClone + Send + Sync + Debug {
    /// Executes a query using the given executor and database connection.
    async fn execute(
        &self,
        method: HttpMethod,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        params: ExecuteParams,
    ) -> Result<ExecuteOutput, RequestError>;
}

#[derive(Debug)]
pub enum ExecuteParams {
    StringMap(HashMap<CompactString, Option<CompactString>>),
    Any(Box<dyn Any + Send + Sync>),
}

pub enum ExecuteOutput {
    Json(serde_json::Value),
    Any(Box<dyn Encode<Output = Bytes> + Send + Sync>),
}
