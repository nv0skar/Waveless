// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct SignUpCaptured;

impl Service<RequestParamsExtractorRequest> for SignUpCaptured {
    type Response = ExecuteOutput;

    type Error = RequestError;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[instrument(skip_all)]
    fn call(&mut self, cx: RequestParamsExtractorRequest) -> Self::Future {
        let future: Pin<_> = Box::pin(async move {
            let (headers, _, request_params, _) = cx;

            let request_params = request_params
                .iter()
                .filter_map(|entry| {
                    if let (key, ExecuteParamValue::Client(Some(value))) = entry {
                        Some((key.to_owned(), value.to_owned()))
                    } else {
                        None
                    }
                })
                .collect();

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

            // Check whether signup is enabled for the current build.
            if !auth_config.allow_signup() {
                Err(RequestError::Other(anyhow!("Signup is disabled for the current build.")))?;
            }

            let databases = DATABASES_CONNS.get().unwrap();

            let auth_method = {
                if auth_config.backends().len() == 1 {
                    auth_config.backends().first().unwrap()
                } else if let Some(auth_backend_name) = headers.get("AuthenticationType") {
                    // Checks for authentication type header if set.
                    if let Ok(auth_backend_name) = auth_backend_name.to_str() {
                        auth_config
                            .backends()
                            .iter()
                            .find(|auth_method| auth_method.name() == auth_backend_name)
                            .ok_or(RequestError::Expected(
                                StatusCode::BAD_REQUEST,
                                "Cannot find the requested authentication method."
                                    .to_compact_string(),
                            ))?
                    } else {
                        return Err(RequestError::Expected(
                            StatusCode::BAD_REQUEST,
                            "Cannot deserialize the `AuthenticationMethod` header."
                                .to_compact_string(),
                        ));
                    }
                } else {
                    return Err(RequestError::Expected(
                        StatusCode::BAD_REQUEST,
                        "No authentication method has been set. HINT: set one using the `AuthenticationMethod` header."
                            .to_compact_string(),
                    ));
                }
            };

            let Ok(auth_db) = databases.search(auth_method.db_id()) else {
                return Err(RequestError::Other(anyhow!(
                    "Cannot get the database connection for '{}'.",
                    auth_method.db_id().unwrap_or("main".to_compact_string())
                )));
            };

            // Create a new user.
            match auth_method.new(auth_db, request_params).await {
                Ok(user_id) => {
                    // Create a new session.
                    let session_method = auth_config.session();

                    let Ok(session_db) = databases.search(session_method.db_id()) else {
                        return Err(RequestError::Other(anyhow!(
                            "Cannot get the database connection for '{}'.",
                            session_method.db_id().unwrap_or("main".to_compact_string())
                        )));
                    };

                    let session_token =
                        session_method
                            .new(session_db, user_id)
                            .await
                            .map_err(|err| {
                                RequestError::Other(anyhow!(
                                    "Cannot check the session token. {}",
                                    err
                                ))
                            })?;


                    let mut headers = HashMap::new();

                    // TODO: should add the secure param to `Set-Cookie`.
                    headers.insert(
                        "Set-Cookie".to_compact_string(),
                        format!(
                            "Authorization={}; SameSite=Lax; {}",
                            session_token,
                            session_method
                                .max_age()
                                .map(|max_age| format!("Max-Age={}", max_age))
                                .unwrap_or_default()
                        )
                        .to_compact_string(),
                    );

                    Ok(ExecuteOutput::Json(
                        Some(headers),
                        json!({
                            "token": session_token
                        }),
                    ))
                },

                Err(err) => Err(RequestError::Other(err))
            }
        }).into();

        future as Self::Future // Actually, this is not an error! https://github.com/rust-lang/rust/issues/92929
    }
}
