// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

/// TODO: add documentation.
#[derive(Clone, Debug)]
pub struct ExecuteHandler;

impl Service<RequestParamsExtractorRequest> for ExecuteHandler {
    type Response = ExecuteOutput;

    type Error = RequestError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    /// Handles endpoints requests.
    fn call(&mut self, cx: RequestParamsExtractorRequest) -> Self::Future {
        Box::pin(async move {
            let (_, endpoint, request_params) = cx;

            // Retrieves the endpoint's target database.
            let database_id = endpoint.target_database();

            let db_conn = DATABASES_CONNS
                .get()
                .unwrap()
                .search(database_id.to_owned())?;

            // Executes request.
            let Some(execute_strategy) = endpoint.execute() else {
                return Err(RequestError::Expected(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("The route doesn't have any executor defined. HINT: Go to your project's endpoints folder and check that '{}' has an executor set.", endpoint.id()).to_compact_string(),
                ));
            };

            execute_strategy
                .execute(
                    *endpoint.method(),
                    db_conn,
                    ExecuteInput::new(request_params, None),
                )
                .await
        })
    }
}
