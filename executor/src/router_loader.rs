// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// Retrieves the endpoints's router from the `ROUTER`'s `OnceCell` or panics if not present.
pub fn router() -> Result<&'static EndpointRouter> {
    match ROUTER.get() {
        Some(router) => Ok(router),
        None => {
            panic!("The `ROUTER` global is not set.")
        }
    }
}

/// Loads all the build's endpoints into the router using the `BUILD` global.
pub async fn load_router() -> Result<EndpointRouter> {
    let _build_lock = runtime_build::build().await?;

    let build = _build_lock.read().await;

    let endpoint_routes = DashMap::<HttpMethod, Router<Endpoint>>::new();

    for endpoint in build.endpoints().inner() {
        let mut full_route = PathBuf::new();
        full_route.push(build.server_settings().api_prefix().trim_matches('/'));
        if let Some(prefix) = endpoint.version() {
            full_route.push(prefix.trim_matches('/'));
        }
        full_route.push(endpoint.route().trim_matches('/'));

        if let Some(mut router) = endpoint_routes.get_mut(endpoint.method()) {
            router.insert(full_route.display().to_string(), endpoint.to_owned())?;
        } else {
            let mut new_router = Router::new();
            new_router.insert(full_route.display().to_string(), endpoint.to_owned())?;
            let _ = endpoint_routes.insert(endpoint.method().to_owned(), new_router);
        }
    }
    Ok(endpoint_routes)
}
