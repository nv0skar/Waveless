// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

pub type RequestParamsExtractorRequest = (
    HeaderMap,
    Endpoint,
    HashMap<CompactString, ParamValue>,
    Bytes,
);

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct RequestParamsExtractor<S>
where
    S: Service<RequestParamsExtractorRequest, Response = ExecuteResponse, Error = RequestError>,
{
    inner: S,
}

pub struct RequestParamsExtractorLayer;

impl<S> Layer<S> for RequestParamsExtractorLayer
where
    S: Service<RequestParamsExtractorRequest, Response = ExecuteResponse, Error = RequestError>,
{
    type Service = RequestParamsExtractor<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestParamsExtractor { inner }
    }
}

impl<S> Service<RouterRequest> for RequestParamsExtractor<S>
where
    S: Service<RequestParamsExtractorRequest, Response = ExecuteResponse, Error = RequestError>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, cx: RouterRequest) -> Self::Future {
        let mut inner = self.inner.to_owned();

        Box::pin(async move {
            let (request, Some((mut request_params, endpoint))) = cx else {
                panic!("Unexpected behaviour");
            };

            let mut request_body = Bytes::new();

            let headers = request.headers().to_owned();

            // Searches for query params.
            if let Some(queries) = request.uri().query() {
                let queries = queries.split('&').map(|elem| {
                    elem.split_once('=')
                        .ok_or(anyhow!("Cannot parse request's query."))
                        .unwrap()
                });
                if *endpoint.capture_all_params() {
                    for (key, value) in queries {
                        request_params.insert(key.into(), ParamValue::Client(Some(value.into())));
                    }
                } else {
                    for key in endpoint.query_params() {
                        let mut owned_iterator = queries.to_owned();
                        match owned_iterator.find(|elem| elem.0 == key) {
                            Some((key, value)) => request_params
                                .insert(key.into(), ParamValue::Client(Some(value.into()))),
                            None => request_params
                                .insert(key.to_compact_string(), ParamValue::Client(None)),
                        };
                    }
                }
            }

            // Searches for body params.
            if !endpoint.body_params().is_empty() || *endpoint.capture_all_params() {
                request_body = CheapVec::from_vec(
                    request
                        .collect()
                        .await
                        .map_err(|err| {
                            RequestError::Expected(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Cannot get request's body. {}", err).into(),
                            )
                        })?
                        .to_bytes()
                        .to_vec(),
                );

                if !request_body.is_empty() {
                    let Ok(json_body) = (match request_body.is_empty() {
                        true => Ok(serde_json::Value::Array(vec![])),
                        false => serde_json::from_slice::<serde_json::Value>(
                            request_body.iter().as_slice(),
                        ),
                    }) else {
                        return Err(RequestError::Expected(
                            StatusCode::BAD_REQUEST,
                            "Invalid request's body. The provided JSON's format is unsupported."
                                .into(),
                        ));
                    };

                    if *endpoint.capture_all_params() {
                        for (key, value) in json_body.as_object().ok_or(RequestError::Expected(
                            StatusCode::BAD_REQUEST,
                            "Cannot extract the parameters from the request's body.".into(),
                        ))? {
                            request_params.insert(
                                key.into(),
                                ParamValue::Client(Some(
                                    value
                                        .as_str()
                                        .map(|s| s.to_string())
                                        .unwrap_or(value.to_string())
                                        .into(),
                                )),
                            );
                        }
                    } else {
                        for key in endpoint.body_params() {
                            let value = {
                                match json_body
                                    .as_object()
                                    .ok_or(RequestError::Expected(
                                        StatusCode::BAD_REQUEST,
                                        "Cannot extract the parameters from the request's body."
                                            .into(),
                                    ))?
                                    .get(key.as_str())
                                {
                                    Some(res) => Some(
                                        res.as_str()
                                            .map(|s| s.to_string())
                                            .unwrap_or(res.to_string())
                                            .into(),
                                    ),
                                    None => None,
                                }
                            };

                            request_params.insert(key.to_owned(), ParamValue::Client(value));
                        }
                    }
                } else if !endpoint.body_params().is_empty() {
                    return Err(RequestError::Expected(
                        StatusCode::BAD_REQUEST,
                        "The request's body for this endpoint cannot be empty.".into(),
                    ));
                }
            }

            inner
                .call((headers, endpoint, request_params, request_body))
                .await
        })
    }
}
