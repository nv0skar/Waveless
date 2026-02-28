// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

pub const LOGIN_ENDPOINT_ID: &str = "Login";

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct AuthCapture<S>
where
    S: Service<RequestParamsExtractorRequest, Response = ExecuteOutput, Error = RequestError>,
{
    inner: S,
}

pub struct AuthCaptureLayer;

impl<S> Layer<S> for AuthCaptureLayer
where
    S: Service<RequestParamsExtractorRequest, Response = ExecuteOutput, Error = RequestError>,
{
    type Service = AuthCapture<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthCapture { inner }
    }
}

impl<S> Service<RequestParamsExtractorRequest> for AuthCapture<S>
where
    S: Service<RequestParamsExtractorRequest, Response = ExecuteOutput, Error = RequestError>
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

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, cx: RequestParamsExtractorRequest) -> Self::Future {
        let mut inner = self.inner.to_owned();

        Box::pin(async move {
            let (headers, endpoint, request_params, request_body) = cx;

            match endpoint.id().as_str() {
                LOGIN_ENDPOINT_ID => {
                    LoginCaptured
                        .call((headers, endpoint, request_params, request_body))
                        .await
                }
                _ => {
                    inner
                        .call((headers, endpoint, request_params, request_body))
                        .await
                }
            }
        })
    }
}
