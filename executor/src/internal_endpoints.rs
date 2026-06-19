// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

pub const LOGIN_ENDPOINT_ID: &str = "Login";
pub const SIGNUP_ENDPOINT_ID: &str = "SignUp";
pub const LOGOUT_ENDPOINT_ID: &str = "Logout";
pub const LOGOUT_ALL_ENDPOINT_ID: &str = "LogoutALl";

/// Internal endpoints provided by the executor.
pub const INTERNAL_ENDPOINTS: LazyCell<[(InternalEndpointKind, Endpoint); 4]> = LazyCell::new(
    || {
        [
            (
                InternalEndpointKind::Authentication,
                EndpointBuilder::default()
                    .id(LOGIN_ENDPOINT_ID.to_compact_string())
                    .route("login".to_compact_string())
                    .method(HttpMethod::Post)
                    .version("internal".to_compact_string())
                    .description("Login a user capturing all parameters and forwarding them to the underlying authentication method.".to_compact_string())
                    .capture_all_params(true)
                    .auto_generated(true)
                    .build()
                    .unwrap()
            ),
            (
                InternalEndpointKind::Authentication,
                EndpointBuilder::default()
                    .id(SIGNUP_ENDPOINT_ID.to_compact_string())
                    .route("signup".to_compact_string())
                    .method(HttpMethod::Post)
                    .version("internal".to_compact_string())
                    .description("Create a new user capturing all parameters and forwarding them to the underlying authentication method.".to_compact_string())
                    .capture_all_params(true)
                    .auto_generated(true)
                    .build()
                    .unwrap()
            ),
            (
                InternalEndpointKind::Authentication,
                EndpointBuilder::default()
                    .id(LOGOUT_ENDPOINT_ID.to_compact_string())
                    .route("logout".to_compact_string())
                    .method(HttpMethod::Get)
                    .version("internal".to_compact_string())
                    .description("Invalidate the current authorization token.".to_compact_string())
                    .require_auth(true)
                    .inject_user_id(true)
                    .auto_generated(true)
                    .build()
                    .unwrap()
            ),
            (
                InternalEndpointKind::Authentication,
                EndpointBuilder::default()
                    .id(LOGOUT_ALL_ENDPOINT_ID.to_compact_string())
                    .route("logout/all".to_compact_string())
                    .method(HttpMethod::Get)
                    .version("internal".to_compact_string())
                    .description("Invalidate all the authorization tokens of the current user.".to_compact_string())
                    .require_auth(true)
                    .inject_user_id(true)
                    .auto_generated(true)
                    .build()
                    .unwrap()
            )
        ]
    },
);

/// Specifies the kind of an internal endpoint.
#[derive(Debug)]
pub enum InternalEndpointKind {
    Authentication,
    Other,
}
