// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

pub type RequestParamsExtractorRequest = (Endpoint, HashMap<CompactString, Option<CompactString>>);

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct RequestParamsExtractor<S>
where
    S: Service<RequestParamsExtractorRequest, Error = RequestError>,
{
    inner: S,
}

pub struct RequestParamsExtractorLayer;

impl<S> Layer<S> for RequestParamsExtractorLayer
where
    S: Service<RequestParamsExtractorRequest, Error = RequestError>,
{
    type Service = RequestParamsExtractor<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestParamsExtractor { inner }
    }
}

impl<S> Service<RouterRequest> for RequestParamsExtractor<S>
where
    S: Service<RequestParamsExtractorRequest, Error = RequestError> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    /// Handles endpoints requests.
    fn call(&mut self, cx: RouterRequest) -> Self::Future {
        let mut inner = self.inner.to_owned();

        Box::pin(async move {
            let (request, Some((mut request_params, endpoint))) = cx else {
                panic!("Unexpected behaviour");
            };

            // Searches for query params.
            if let Some(queries) = request.uri().query() {
                let queries = queries.split('&').map(|elem| {
                    elem.split_once('=')
                        .ok_or(anyhow!("Cannot parse request's query."))
                        .unwrap()
                });
                for key in endpoint.query_params() {
                    let mut owned_iterator = queries.to_owned();
                    match owned_iterator.find(|elem| elem.0 == key) {
                        Some((key, value)) => request_params
                            .insert(key.to_compact_string(), Some(value.to_compact_string())),
                        None => request_params.insert(key.to_compact_string(), None),
                    };
                }
            }

            // Searches for body params.
            if !endpoint.body_params().is_empty() {
                let req_body = request
                    .collect()
                    .await
                    .map_err(|err| {
                        RequestError::Expected(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Cannot get request's body. {}", err).to_compact_string(),
                        )
                    })?
                    .to_bytes();

                if req_body.is_empty() {
                    return Err(RequestError::Expected(
                        StatusCode::BAD_REQUEST,
                        "The request's body for this endpoint cannot be empty.".to_compact_string(),
                    ));
                }

                let Ok(json_body) =
                    serde_json::from_slice::<serde_json::Value>(req_body.iter().as_slice())
                else {
                    return Err(RequestError::Expected(
                        StatusCode::BAD_REQUEST,
                        "Invalid request's body. The provided JSON's format is unsupported."
                            .to_compact_string(),
                    ));
                };

                for key in endpoint.body_params() {
                    let value = {
                        match json_body.as_object().unwrap().get(key.as_str()) {
                            Some(res) => Some(
                                res.as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or(res.to_string())
                                    .to_compact_string(),
                            ),
                            None => None,
                        }
                    };

                    request_params.insert(key.to_owned(), value);
                }
            }

            inner.call((endpoint, request_params)).await
        })
    }
}
