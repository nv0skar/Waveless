// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// Endpoint handler wrapper that serializes response into JSON.
/// TODO: Convert this into a service layer.
pub async fn handle_endpoint(request: Request<Incoming>) -> Result<Response<String>> {
    let response = Response::builder()
        .header("Content-Type", "application/json; charset=utf-8")
        .header(
            "Cache-Control",
            format!(
                "max-age={}",
                *build_loader::build()?.server_settings().http_cache_time() as u32
            ),
        );

    match try_handle_endpoint(request).await {
        Ok(value) => Ok(response
            .status(200)
            .body(serde_json::to_string_pretty(&value)?)?),
        Err(err) => Ok(response
            .status({
                match err {
                    ConnHandlerError::Expected(status, _) => status,
                    ConnHandlerError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            })
            .body(serde_json::to_string_pretty(&json!({
                "error": match err {
                    ConnHandlerError::Expected(_, err) => err,
                    ConnHandlerError::Other(err) => format!("Unexpected error: {}", err).to_compact_string(),
                }
            }))?)?),
    }
}

/// Handles endpoints requests.
#[instrument(skip_all)]
pub async fn try_handle_endpoint(
    request: Request<Incoming>,
) -> Result<serde_json::Value, ConnHandlerError> {
    info!(
        "{} request at {} from {}",
        request.method(),
        request.uri().path(),
        request
            .headers()
            .get("host")
            .map(|val| val.to_str().unwrap_or_default())
            .unwrap_or_default()
    );

    let method = endpoint::HttpMethod::from(request.method().as_str());

    // Extracts the route from the method-aware router.
    let Some(router) = router_loader::router()?.get(&method) else {
        return Err(ConnHandlerError::Expected(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("There is no route that accepts {}.", method).to_compact_string(),
        ));
    };

    let route = request.uri().path().trim_matches('/').to_owned();

    let Ok(endpoint_def) = router.at(&route) else {
        return Err(ConnHandlerError::Expected(
            StatusCode::NOT_FOUND,
            format!("Route '{}' is not defined. HINT: Go to your project's endpoints folder and check the endpoint's routes.", route).to_compact_string(),
        ));
    };

    // Retrieves the endpoint's target database.
    let database_id = endpoint_def.value.target_database();

    let dbs_conns = DATABASES_CONNS
        .get()
        .unwrap()
        .search(database_id.to_owned())?;

    // Checks for path params, query params and body params.
    let mut request_params = HashMap::<CompactString, CompactString>::new();

    for (key, value) in endpoint_def.params.iter() {
        request_params.insert(key.to_compact_string(), value.to_compact_string());
    }

    if let Some(queries) = request.uri().query() {
        let queries = queries.split('&');
        for query in queries {
            let (key, value) = query
                .split_once('=')
                .ok_or(anyhow!("Cannot parse request's query."))?;
            request_params.insert(key.to_compact_string(), value.to_compact_string());
        }
    }

    if !endpoint_def.value.body_params().is_empty() {
        let req_body = request
            .collect()
            .await
            .map_err(|err| {
                ConnHandlerError::Expected(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Cannot get request's body. {}", err).to_compact_string(),
                )
            })?
            .to_bytes();

        if req_body.is_empty() {
            return Err(ConnHandlerError::Expected(
                StatusCode::BAD_REQUEST,
                "The request's body for this endpoint cannot be empty.".to_compact_string(),
            ));
        }

        let Ok(json_body) = serde_json::from_slice::<serde_json::Value>(req_body.iter().as_slice())
        else {
            return Err(ConnHandlerError::Expected(
                StatusCode::BAD_REQUEST,
                "Invalid request's body. The provided JSON's format is unsupported."
                    .to_compact_string(),
            ));
        };

        for key in endpoint_def.value.body_params() {
            let Some(value) = json_body.as_object().unwrap().get(key.as_str()) else {
                return Err(ConnHandlerError::Expected(
                    StatusCode::BAD_REQUEST,
                    "Invalid request's body. Cannot find all the required parameters."
                        .to_compact_string(),
                ));
            };

            request_params.insert(
                key.to_owned(),
                value
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or(value.to_string())
                    .to_compact_string(),
            );
        }
    }

    // Executes request.
    let Some(execute_strategy) = endpoint_def.value.execute() else {
        return Err(ConnHandlerError::Expected(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The route wasn't managed by any of the request handlers.".to_compact_string(),
        ));
    };

    let executor = execute::ExecuteExt::new(execute_strategy.to_owned());

    Ok(executor.execute(dbs_conns, request_params).await?)
}

#[derive(Error, Debug)]
pub enum ConnHandlerError {
    #[error("Request error.")]
    Expected(StatusCode, CompactString),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
