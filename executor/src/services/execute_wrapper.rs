// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct ExecuteWrapper<S>
where
    S: Service<RouterRequest, Error = RequestError>,
{
    inner: S,
}

pub struct ExecuteWrapperLayer;

impl<S> Layer<S> for ExecuteWrapperLayer
where
    S: Service<RouterRequest, Response = ExecuteOutput, Error = RequestError>,
{
    type Service = ExecuteWrapper<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ExecuteWrapper { inner }
    }
}

impl<S> Service<RouterRequest> for ExecuteWrapper<S>
where
    S: Service<RouterRequest, Response = ExecuteOutput, Error = RequestError>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = Response<String>;

    type Error = Infallible;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|_| unreachable!())
    }

    /// Handles endpoints requests.
    #[instrument(skip_all)]
    fn call(&mut self, cx: RouterRequest) -> Self::Future {
        let (req, params) = cx;

        info!(
            "{} request at {} from {}",
            req.method(),
            req.uri().path(),
            req.headers()
                .get("host")
                .map(|val| val.to_str().unwrap_or_default())
                .unwrap_or_default()
        );

        let fut = self.inner.call((req, params));

        Box::pin(async move {
            let response = Response::builder()
                .header("Content-Type", "application/json; charset=utf-8")
                .header(
                    "Cache-Control",
                    format!(
                        "max-age={}",
                        (*RuntimeCx::acquire()
                            .build()
                            .read()
                            .await
                            .executor()
                            .http_cache_time()) as u32
                    ),
                );

            match fut.await {
                Ok(output) => Ok(response
                    .status(200)
                    .body(match output {
                        ExecuteOutput::Json(val) => serde_json::to_string_pretty(&val).unwrap(),
                        ExecuteOutput::Any(val) => serde_json::to_string_pretty(&json!({
                            "data": val.encode().unwrap()
                        })).unwrap(),
                    }).unwrap()
                ),
                Err(err) => Ok(response
                    .status({
                        match err {
                            RequestError::Expected(status, _) => status,
                            RequestError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
                        }
                    })
                    .body(serde_json::to_string_pretty(&json!({
                        "error": match err {
                            RequestError::Expected(_, err) => err,
                            RequestError::Other(err) => format!("Unexpected error: {}", err).to_compact_string(),
                        }
                    })).unwrap()).unwrap()
                ),
            }
        })
    }
}
