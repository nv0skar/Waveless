// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct UpgradeCapture<S>
where
    S: Service<RequestCx, Response = HttpResponse, Error = RequestError>,
{
    inner: S,
}

pub struct UpgradeCaptureLayer;

impl<S> Layer<S> for UpgradeCaptureLayer
where
    S: Service<RequestCx, Response = HttpResponse, Error = RequestError>,
{
    type Service = UpgradeCapture<S>;

    fn layer(&self, inner: S) -> Self::Service {
        UpgradeCapture { inner }
    }
}

impl<S> Service<RequestCx> for UpgradeCapture<S>
where
    S: Service<RequestCx, Response = HttpResponse, Error = RequestError> + Clone + Send + 'static,
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

    fn call(&mut self, cx: RequestCx) -> Self::Future {
        let mut inner = self.inner.to_owned();

        Box::pin(async move {
            let RequestCx { endpoint, .. } = &cx;

            match endpoint.id().as_str() {
                CONN_UPGRADE_WEBSOCKETS_ENDPOINT_ID => WebSocketsSvc.call(cx).await,
                _ => inner.call(cx).await,
            }
        })
    }
}
