// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod request_cx;

use request_cx::*;

use crate::*;

use databases::*;

/// Generic methods trait to handle requests to the endpoints.
#[typetag::serde]
#[async_trait]
pub trait AnyHttpExecute: Any + BoxedAny + DynClone + Send + Sync + Debug {
    /// Executes a query using the given executor and database connection.
    async fn execute(
        &self,
        cx: RequestCx,
        db_conn: Arc<dyn AnyDatabaseConnection>,
    ) -> Result<HttpResponse, RequestError>;
}

/// TODO: add documentation.
#[derive(Clone, Debug)]
pub enum ParamValue {
    Internal(CompactString),
    Client(Option<CompactString>),
}

#[derive(Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct HttpResponse {
    status: StatusCode,
    headers: Option<HashMap<CompactString, CompactString>>,
    body: Option<BodyValue>,
}

impl HttpResponse {
    pub fn new(
        headers: Option<HashMap<CompactString, CompactString>>,
        body: Option<BodyValue>,
    ) -> Self {
        Self {
            status: StatusCode::OK,
            headers,
            body,
        }
    }

    // Cannot change the default name of `Constructor`.
    pub fn new_with_status(
        status: StatusCode,
        headers: Option<HashMap<CompactString, CompactString>>,
        body: Option<BodyValue>,
    ) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }
}

pub enum BodyValue {
    Json(serde_json::Value),
    Any(Box<dyn Encode<Output = Bytes> + Send + Sync>),
}
