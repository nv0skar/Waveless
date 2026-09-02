// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use waveless_commons::databases::DatabaseConsumer;

use crate::*;

/// TODO: add documentation.
#[derive(Clone, Debug)]
pub struct ExecuteHandler;

impl Service<RequestCx> for ExecuteHandler {
    type Response = HttpResponse;

    type Error = RequestError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    /// Handles endpoints requests.
    fn call(&mut self, cx: RequestCx) -> Self::Future {
        let future: Pin<_> = Box::pin(async move {
            let RequestCx { endpoint, .. } = &cx;

            // Retrieves the endpoint's target database.
            let db_conns = endpoint.get_db_handle()?;

            // Force the endpoint to have the HTTP target.
            let ExecutionTarget::Http(http_target) = endpoint.execution_target().to_owned() else {
                unreachable!()
            };

            // Executes request.
            let Some(execute_strategy) = http_target.execute() else {
                return Err(RequestError::Expected(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("The route doesn't have any executor defined. HINT: Go to your project's endpoints folder and check that '{}' has an executor set.", endpoint.id()).into(),
                ));
            };

            execute_strategy
                .execute(
                    cx,
                    db_conns,
                )
                .await
        }).into();

        future as Self::Future // `rust-analyzer` complains here.
    }
}
