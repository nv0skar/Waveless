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
        input: ExecuteInput,
    ) -> Result<ExecuteOutput, RequestError>;
}

/// TODO: add documentation.
#[derive(Clone, Constructor, Getters, Debug)]
#[getset(get = "pub")]
pub struct ExecuteInput {
    /// Note that by default, path params, query params, and JSON
    /// formatted bodies are serialized (by default) to this field.
    params: HashMap<CompactString, ExecuteParamValue>,
    value: Bytes,
}

/// TODO: add documentation.
#[derive(Clone, Debug)]
pub enum ExecuteParamValue {
    Internal(CompactString),
    Client(Option<CompactString>),
}

pub enum ExecuteOutput {
    Json(
        Option<HashMap<CompactString, CompactString>>,
        serde_json::Value,
    ),
    Any(Box<dyn Encode<Output = Bytes> + Send + Sync>),
}
