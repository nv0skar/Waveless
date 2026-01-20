// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use http_body_util::BodyExt;

use crate::*;

// /// Endpoint handler wrapper that serializes response into JSON
// pub async fn handle_endpoint(request: Request<hyper::body::Incoming>) -> Result<Response<String>> {
//     match try_handle_endpoint(todo!()).await {
//         Ok(res) => todo!(),
//         Err(err) => err.downcast::<usize>(),
//     };

//     todo!()
// }

/// Handles endpoints requests.
pub async fn try_handle_endpoint(
    request: Request<hyper::body::Incoming>,
) -> Result<Response<String>> {
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

    // Default response headers.
    let response = Response::builder()
        .header("Content-Type", "application/json")
        .header(
            "Cache-Control",
            format!(
                "max-age={}",
                *build_loader::build()?.server_settings().http_cache_time() as u32
            ),
        );

    let method = endpoint::HttpMethod::from(request.method().as_str());

    // Extracts the route from the method-aware router.
    let Some(router) = router_loader::router()?.get(&method) else {
        return Ok(response
            .status(500)
            .body(serde_json::to_string_pretty(&json!({
                "error": format!("There is no route that accepts {}.", method)
            }))?)?);
    };

    let route = request.uri().path().trim_matches('/').to_owned();

    let Ok(endpoint_def) = router.at(&route) else {
        return Ok(response
            .status(404)
            .body(serde_json::to_string_pretty(&json!({
                "error": format!("Route '{}' is not defined.", route),
                "hint": "Go to your project's endpoints folder and check the endpoint's routes."
            }))?)?);
    };

    // Retrieves the endpoint's target database.
    let database_id = endpoint_def.value.target_database();

    let pool = DATABASE_POOLS
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
        let req_body = request.collect().await?.to_bytes();

        if req_body.is_empty() {
            return Ok(response
                .status(404)
                .body(serde_json::to_string_pretty(&json!({
                    "error": "This endpoint expects a body."
                }))?)?);
        }

        let Ok(json_body) = serde_json::from_slice::<serde_json::Value>(req_body.iter().as_slice())
        else {
            return Ok(response
                .status(404)
                .body(serde_json::to_string_pretty(&json!({
                    "error": "Invalid body.",
                }))?)?);
        };

        for key in endpoint_def.value.body_params() {
            let Some(value) = json_body.as_object().unwrap().get(key.as_str()) else {
                return Ok(response
                    .status(404)
                    .body(serde_json::to_string_pretty(&json!({
                        "error": "Invalid body: cannot find requrired parameter."
                    }))?)?);
            };
            request_params.insert(key.to_owned(), format!("{}", value).to_compact_string());
        }
    }

    // Request handling.
    if let Some(executor) = endpoint_def.value.executor() {
        match executor {
            endpoint::Execute::MySQL { query } => {
                let databases::AnyDatabaseConnection::MySQL(mysql_pool) = pool;

                // Replaces Waveless' query's parameters placeholders with MySQL's ones.
                let params_order = query
                    .trim_start_matches(|c| c != '{')
                    .split('{')
                    .map(|sub| sub.split_once('}').unwrap_or_default().0)
                    .filter(|sub| !sub.is_empty())
                    .collect::<CheapVec<&str>>();

                let mut query_params = CheapVec::new();

                for param_id in params_order {
                    let Some(value) = request_params.get(&param_id.to_compact_string()) else {
                        return Ok(response.status(400).body(serde_json::to_string_pretty(
                            &json!({
                                "error": "Missing data."
                            }),
                        )?)?);
                    };

                    query_params.push(sea_orm::Value::from(value.to_string()));
                }

                let mysql_query = query
                    .split('{')
                    .map(|sub| {
                        if sub.contains('}') {
                            sub.trim_start_matches(|c| c != '}').replace('}', "?")
                        } else {
                            sub.to_string()
                        }
                    })
                    .collect::<CompactString>();

                let res = mysql_pool
                    .query_all(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        mysql_query.to_string(),
                        query_params,
                    ))
                    .await?;

                let mut rows = CheapVec::new();

                for row in res {
                    rows.push(JsonValue::from_query_result(&row, "")?);
                }

                return Ok(response.body(serde_json::to_string_pretty(&rows)?)?);
            }
            endpoint::Execute::Hook { .. } => {
                todo!("Custom endpoint hooks aren't implemented yet.")
            }
        }
    }

    Ok(response
        .status(500)
        .body(serde_json::to_string_pretty(&json!({
            "error": "The route wasn't managed by any of the request handlers."
        }))?)?)
}
