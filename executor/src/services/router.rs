// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

pub type RouterRequest = (
    Request<Incoming>,
    Option<(HashMap<CompactString, ExecuteParamValue>, Endpoint)>,
);

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct RouterService<S>
where
    S: Service<RouterRequest, Error = Infallible>,
{
    endpoints: S,
    frontend: S,
}

impl<S> Service<Request<Incoming>> for RouterService<S>
where
    S: Service<RouterRequest, Error = Infallible> + Send + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.endpoints.poll_ready(cx).is_ready() && self.frontend.poll_ready(cx).is_ready() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn call(&mut self, request: Request<Incoming>) -> Self::Future {
        let method = HttpMethod::from(request.method().as_str());

        // Extracts the route from the method-aware router.
        let Some(router) = RuntimeCx::acquire().router().get(&method) else {
            // TODO: handle from here to the frontend.
            // If a path isn't found in the frontend or if it'S
            // disabled return the JSON INTERNAL_SERVER_ERROR error message:
            // return Box::pin(async move {
            //     Err(RequestError::Expected(
            //         StatusCode::INTERNAL_SERVER_ERROR,
            //         format!("There is no route that accepts {}.", method).to_compact_string(),
            //     ))
            // });
            todo!("Frontend not implemented yet.");
        };

        // Tries to match the route.
        let route = request.uri().path().trim_matches('/').to_owned();

        let Ok(matched) = router.at(&route) else {
            // TODO: handle from here to the frontend.
            // If a path isn't found in the frontend or if it'S
            // disabled return the JSON NOT_FOUND error message:
            // return Err(RequestError::Expected(
            //     StatusCode::NOT_FOUND,
            //     format!("Route '{}' is not defined. HINT: Go to your project's endpoints folder and check the endpoint's routes.", route).to_compact_string(),
            // ));
            todo!("Frontend not implemented yet.");
        };

        // Extracts the path's params.
        let mut path_params = HashMap::<CompactString, ExecuteParamValue>::new();

        for (key, value) in matched.params.iter() {
            path_params.insert(
                key.to_compact_string(),
                ExecuteParamValue::Client(Some(value.to_compact_string())),
            );
        }

        self.endpoints
            .call((request, Some((path_params, matched.value.to_owned()))))
    }
}
