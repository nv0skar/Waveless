// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// TODO: add documentation.
#[derive(Clone, Constructor, Debug)]
pub struct SessionWatchdog<S>
where
    S: Service<RequestParamsExtractorRequest, Response = ExecuteOutput, Error = RequestError>,
{
    inner: S,
}

pub struct SessionWatchdogLayer;

impl<S> Layer<S> for SessionWatchdogLayer
where
    S: Service<RequestParamsExtractorRequest, Response = ExecuteOutput, Error = RequestError>,
{
    type Service = SessionWatchdog<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionWatchdog { inner }
    }
}

impl<S> Service<RequestParamsExtractorRequest> for SessionWatchdog<S>
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

    #[instrument(skip_all)]
    fn call(&mut self, cx: RequestParamsExtractorRequest) -> Self::Future {
        let mut inner = self.inner.to_owned();

        Box::pin(async move {
            let (headers, endpoint, mut request_params, request_body) = cx;

            // Checks whether the current endpoint requires auth.
            if *endpoint.require_auth() {
                let _build_lock = RuntimeCx::acquire().build();

                let auth_config = _build_lock
                    .read()
                    .await
                    .config()
                    .authentication()
                    .to_owned();

                let Some(auth_config) = auth_config else {
                    // TODO: the compiler should fail when including endpoints
                    // that require authentication while not having
                    // authentication set for the project.
                    return Err(RequestError::Other(anyhow!(
                        "Endpoint '{}' requires auth but authentication is not set for this build.",
                        endpoint.id()
                    )));
                };

                // Loads the session's method's and role's method's databases.
                let session_method = auth_config.session();

                let role_method = auth_config.role();

                let databases = DATABASES_CONNS.get().unwrap();

                let Ok(session_db) = databases.search(session_method.db_id()) else {
                    return Err(RequestError::Other(anyhow!(
                        "Cannot get the database connection for '{}'.",
                        session_method.db_id().unwrap_or("main".to_compact_string())
                    )));
                };

                // Check the session.
                let token = match (headers.get("Authorization"), headers.get("Cookie")) {
                    (Some(auth_header), _) => {
                        if let Ok(token) = auth_header.to_str() {
                            Some(token)
                        } else {
                            // TODO: future connections from the same origin
                            // may be throttled.
                            return Err(RequestError::Expected(
                                StatusCode::BAD_REQUEST,
                                "Malformed auth header.".to_compact_string(),
                            ));
                        }
                    }
                    (_, Some(cookie_header)) => {
                        if let Ok(cookies) = cookie_header.to_str() {
                            let cookies = cookies
                                .trim()
                                .split(';')
                                .map(|cookie| cookie.split_once('='));
                            let authorization_cookie = cookies
                                .filter(|opt| {
                                    opt.map(|(name, _)| name.to_lowercase() == "authorization")
                                        .unwrap_or(false)
                                })
                                .flatten()
                                .next();
                            if let Some((_, token)) = authorization_cookie {
                                Some(token)
                            } else {
                                None
                            }
                        } else {
                            // TODO: future connections from the same origin
                            // may be throttled.
                            return Err(RequestError::Expected(
                                StatusCode::BAD_REQUEST,
                                "Malformed cookie header.".to_compact_string(),
                            ));
                        }
                    }
                    _ => None,
                };

                let Some(token) = token else {
                    return Err(RequestError::Expected(
                        StatusCode::UNAUTHORIZED,
                        format!("'{}' requires authentication.", endpoint.id()).to_compact_string(),
                    ));
                };

                let session_check = session_method
                    .check(session_db, token.to_compact_string())
                    .await
                    .map_err(|err| {
                        RequestError::Other(anyhow!("Cannot check the session token. {}", err))
                    })?;

                match session_check {
                    Some(user_id) => {
                        // Inject user id if required.
                        if *endpoint.inject_user_id() {
                            request_params.insert(
                                "user_id".to_compact_string(),
                                ExecuteParamValue::Internal(user_id.to_compact_string()),
                            );
                        }
                        if endpoint.allowed_roles().is_empty() {
                            inner
                                .call((headers, endpoint, request_params, request_body))
                                .await
                        } else {
                            let Some(role_method) = role_method else {
                                // TODO: the compiler should fail when including endpoints
                                // that require roles while not having
                                // roles set for the project.
                                return Err(RequestError::Other(anyhow!(
                                    "Endpoint '{}' requires roles authentication but they are not set for this build.",
                                    endpoint.id()
                                )));
                            };

                            let Ok(role_db) = databases.search(role_method.db_id()) else {
                                return Err(RequestError::Other(anyhow!(
                                    "Cannot get the database connection for '{}'.",
                                    session_method.db_id().unwrap_or("main".to_compact_string())
                                )));
                            };

                            let Ok(role_check) = role_method.get(role_db, user_id).await else {
                                return Err(RequestError::Other(anyhow!(
                                    "Cannot check the user's role."
                                )));
                            };

                            let Some(role) = role_check else {
                                return Err(RequestError::Expected(
                                    StatusCode::UNAUTHORIZED,
                                    "Current user does not have any role.".to_compact_string(),
                                ));
                            };

                            if endpoint.allowed_roles().contains(&role.to_lowercase()) {
                                inner
                                    .call((headers, endpoint, request_params, request_body))
                                    .await
                            } else {
                                return Err(RequestError::Expected(
                                    StatusCode::UNAUTHORIZED,
                                    "Current user does not have any of the allowed roles."
                                        .to_compact_string(),
                                ));
                            }
                        }
                    }
                    None => Err(RequestError::Expected(
                        StatusCode::UNAUTHORIZED,
                        "Invalid session.".to_compact_string(),
                    )),
                }
            } else {
                inner
                    .call((headers, endpoint, request_params, request_body))
                    .await
            }
        })
    }
}
