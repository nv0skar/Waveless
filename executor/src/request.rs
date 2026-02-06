// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// Endpoint handler wrapper that serializes response into JSON.
/// TODO: Convert this into a service layer.
#[instrument(skip_all)]
pub async fn handle_endpoint(request: Request<Incoming>) -> Result<Response<String>, Infallible> {
    let response = Response::builder()
        .header("Content-Type", "application/json; charset=utf-8")
        .header(
            "Cache-Control",
            format!(
                "max-age={}",
                (*runtime_build::build()
                    .await
                    .unwrap()
                    .read()
                    .await
                    .server_settings()
                    .http_cache_time()) as u32
            ),
        );

    match try_handle_endpoint(request).await {
        Ok(output) => Ok(response
            .status(200)
            .body(match output {
                ExecuteOutput::Json(val) => serde_json::to_string_pretty(&val).unwrap(),
                ExecuteOutput::Any(val) => serde_json::to_string_pretty(&json!({
                    "data": val.encode().unwrap()
                })).unwrap(),
            }).unwrap()),
        Err(err) => Ok(response
            .status({
                match err {
                    RequestError::Expected(status, _) => status,
                    RequestError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
                }
            })
            .body(serde_json::to_string_pretty(&json!({
                "error": match err {
                    RequestError::Expected(_, err) => err,
                    RequestError::Other(err) => format!("Unexpected error: {}", err).to_compact_string(),
                }
            })).unwrap()).unwrap()),
    }
}

/// Handles endpoints requests.
#[instrument(skip_all)]
pub async fn try_handle_endpoint(
    request: Request<Incoming>,
) -> Result<ExecuteOutput, RequestError> {
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

    let method = HttpMethod::from(request.method().as_str());

    // Extracts the route from the method-aware router.
    let Some(router) = router_loader::router()?.get(&method) else {
        return Err(RequestError::Expected(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("There is no route that accepts {}.", method).to_compact_string(),
        ));
    };

    let route = request.uri().path().trim_matches('/').to_owned();

    let Ok(endpoint_def) = router.at(&route) else {
        return Err(RequestError::Expected(
            StatusCode::NOT_FOUND,
            format!("Route '{}' is not defined. HINT: Go to your project's endpoints folder and check the endpoint's routes.", route).to_compact_string(),
        ));
    };

    // Retrieves the endpoint's target database.
    let database_id = endpoint_def.value.target_database();

    let db_conn = DATABASES_CONNS
        .get()
        .unwrap()
        .search(database_id.to_owned())?;

    // Checks for path params, query params and body params.
    let mut request_params = HashMap::<CompactString, Option<CompactString>>::new();

    for (key, value) in endpoint_def.params.iter() {
        request_params.insert(key.to_compact_string(), Some(value.to_compact_string()));
    }

    if let Some(queries) = request.uri().query() {
        let queries = queries.split('&').map(|elem| {
            elem.split_once('=')
                .ok_or(anyhow!("Cannot parse request's query."))
                .unwrap()
        });
        for key in endpoint_def.value.query_params() {
            let mut owned_iterator = queries.to_owned();
            match owned_iterator.find(|elem| elem.0 == key) {
                Some((key, value)) => {
                    request_params.insert(key.to_compact_string(), Some(value.to_compact_string()))
                }
                None => request_params.insert(key.to_compact_string(), None),
            };
        }
    }

    if !endpoint_def.value.body_params().is_empty() {
        let req_body = request
            .collect()
            .await
            .map_err(|err| {
                RequestError::Expected(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Cannot get request's body. {}", err).to_compact_string(),
                )
            })?
            .to_bytes();

        if req_body.is_empty() {
            return Err(RequestError::Expected(
                StatusCode::BAD_REQUEST,
                "The request's body for this endpoint cannot be empty.".to_compact_string(),
            ));
        }

        let Ok(json_body) = serde_json::from_slice::<serde_json::Value>(req_body.iter().as_slice())
        else {
            return Err(RequestError::Expected(
                StatusCode::BAD_REQUEST,
                "Invalid request's body. The provided JSON's format is unsupported."
                    .to_compact_string(),
            ));
        };

        for key in endpoint_def.value.body_params() {
            let value = {
                match json_body.as_object().unwrap().get(key.as_str()) {
                    Some(res) => Some(
                        res.as_str()
                            .map(|s| s.to_string())
                            .unwrap_or(res.to_string())
                            .to_compact_string(),
                    ),
                    None => None,
                }
            };

            request_params.insert(key.to_owned(), value);
        }
    }

    // Executes request.
    let Some(execute_strategy) = endpoint_def.value.execute() else {
        return Err(RequestError::Expected(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The route wasn't managed by any of the request handlers.".to_compact_string(),
        ));
    };

    execute_strategy
        .execute(method, db_conn, ExecuteParams::StringMap(request_params))
        .await
}
