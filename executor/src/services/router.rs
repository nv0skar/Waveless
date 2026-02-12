// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

pub type RouterRequest = (
    Request<Incoming>,
    Option<(HashMap<CompactString, ExecuteParamValue>, Endpoint)>,
);

pub type RouterServiceInner = BoxCloneService<RouterRequest, Response<String>, Infallible>;

/// TODO: add documentation.
#[derive(Clone, Constructor)]
pub struct RouterService<S> {
    endpoints: S,
    frontend: Option<RouterServiceInner>,
}

impl<S> Service<Request<Incoming>> for RouterService<S>
where
    S: Service<RouterRequest, Response = Response<String>, Error = Infallible> + Send + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = Response<String>;

    type Error = Infallible;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let frontend_ready = match &mut self.frontend {
            Some(frontend) => frontend.poll_ready(cx).is_ready(),
            None => true,
        };

        if self.endpoints.poll_ready(cx).is_ready() && frontend_ready {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn call(&mut self, request: Request<Incoming>) -> Self::Future {
        let method = HttpMethod::from(request.method().as_str());

        // Tries to match the route.
        let route = request.uri().path().trim_matches('/').to_owned();

        // Extracts the route from the method-aware router.
        let Some(router) = RuntimeCx::acquire().router().get(&method) else {
            if let Some(mut frontend_inner) = self.frontend.to_owned() {
                return Box::pin(async move {
                    frontend_inner
                        .call((request, None))
                        .await
                        .map_err(|_| unreachable!())
                });
            } else {
                return Box::pin(async move {
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

                    Ok(response
                        .status(404)
                        .body(
                            serde_json::to_string_pretty(&json!({
                                    "error": format!(
                                        "There is no route that accepts {}.",
                                        method
                                    )
                                }
                            ))
                            .unwrap(),
                        )
                        .unwrap())
                });
            }
        };

        let Ok(matched) = router.at(&route) else {
            if let Some(mut frontend_inner) = self.frontend.to_owned() {
                return Box::pin(async move {
                    frontend_inner
                        .call((request, None))
                        .await
                        .map_err(|_| unreachable!())
                });
            } else {
                return Box::pin(async move {
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

                    Ok(response
                            .status(404)
                            .body(serde_json::to_string_pretty(&json!({
                                "error": format!(
                                    "Route '{}' is not defined. HINT: Go to your project's endpoints folder and check the endpoint's routes.",
                                    route
                                )
                            }
                            )).unwrap()
                        ).unwrap()
                    )
                });
            }
        };

        // Extracts the path's params.
        let mut path_params = HashMap::<CompactString, ExecuteParamValue>::new();

        for (key, value) in matched.params.iter() {
            path_params.insert(
                key.to_compact_string(),
                ExecuteParamValue::Client(Some(value.to_compact_string())),
            );
        }

        let endpoint_fut = self.endpoints.call((
            request,
            Some((path_params.to_owned(), matched.value.to_owned())),
        ));

        return Box::pin(async move { endpoint_fut.await.map_err(|_| unreachable!()) });
    }
}
