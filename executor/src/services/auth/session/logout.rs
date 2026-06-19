// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct LogoutCaptured;

impl Service<RequestParamsExtractorRequest> for LogoutCaptured {
    type Response = ExecuteOutput;

    type Error = RequestError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[instrument(skip_all)]
    fn call(&mut self, cx: RequestParamsExtractorRequest) -> Self::Future {
        let future: Pin<_> = Box::pin(async move {
            let (_, endpoint, request_params, _) = cx;

            let auth_config = RuntimeCx::acquire()
                .build()
                .read()
                .await
                .config()
                .authentication()
                .to_owned()
                .ok_or(RequestError::Other(anyhow!(
                    "Authentication is not set for the current build."
                )))?;

            let all_sessions = endpoint.route().split("/").last().unwrap().to_lowercase() == "all";

            let user_id =
                match request_params
                    .get("user_id")
                    .ok_or(RequestError::Other(anyhow!(
                        "Cannot logout as there is no session active.",
                    )))? {
                    ExecuteParamValue::Internal(user_id) => Ok(user_id.to_owned()),
                    _ => Err(RequestError::Expected(
                        StatusCode::FORBIDDEN,
                        "User id injection from the client is forbidden. HINT: if you are debugging your app you can try creating a new session manually.".to_compact_string(),
                    )),
                }?.parse::<UserId>().map_err(|_| RequestError::Other(anyhow!("Cannot convert user id to it's internal representation.")))?;


            let token =
                match request_params
                    .get("token")
                    .unwrap() {
                    ExecuteParamValue::Internal(token) if !all_sessions => Ok(Some(token.to_owned())),
                    ExecuteParamValue::Internal(_) if all_sessions => Ok(None),
                    _ => Err(RequestError::Expected(
                        StatusCode::FORBIDDEN,
                        "Session token injection from the client is forbidden. HINT: if you are debugging your app you can try creating a new session manually.".to_compact_string(),
                    )),
                }?;

            let session_method = auth_config.session();

            let databases = DATABASES_CONNS.get().unwrap();

            let Ok(session_db) = databases.search(session_method.db_id()) else {
                return Err(RequestError::Other(anyhow!(
                    "Cannot get the database connection for '{}'.",
                    session_method.db_id().unwrap_or("main".to_compact_string())
                )));
            };

            session_method.invalidate(session_db, user_id, token).await?;

            Ok(ExecuteOutput::Json(None, json!({})))
        })
        .into();

        future as Self::Future // Actually, this is not an error! https://github.com/rust-lang/rust/issues/92929
    }
}
