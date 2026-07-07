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
        input: ExecuteRequest,
    ) -> Result<ExecuteResponse, RequestError>;
}

/// TODO: add documentation.
#[derive(Clone, Constructor, Getters, MutGetters, Debug)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ExecuteRequest {
    /// Note that by default, path params, query params, and JSON
    /// formatted bodies are serialized (by default) to this field.
    params: HashMap<CompactString, ParamValue>,
    value: Bytes,
}

/// TODO: add documentation.
#[derive(Clone, Debug)]
pub enum ParamValue {
    Internal(CompactString),
    Client(Option<CompactString>),
}

#[derive(Constructor, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ExecuteResponse {
    headers: Option<HashMap<CompactString, CompactString>>,
    body: Option<BodyValue>,
}

pub enum BodyValue {
    Json(serde_json::Value),
    Any(Box<dyn Encode<Output = Bytes> + Send + Sync>),
}
