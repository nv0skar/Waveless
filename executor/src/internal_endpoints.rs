// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

pub const CONN_UPGRADE_WEBSOCKETS_ENDPOINT_ID: &str = "ConnUpgradeWebSockets";

pub const LOGIN_ENDPOINT_ID: &str = "Login";
pub const SIGNUP_ENDPOINT_ID: &str = "SignUp";
pub const LOGOUT_ENDPOINT_ID: &str = "Logout";
pub const LOGOUT_ALL_ENDPOINT_ID: &str = "LogoutAll";

/// Internal endpoints provided by the executor.
pub const INTERNAL_ENDPOINTS: LazyCell<[(InternalEndpointKind, Endpoint); 5]> = LazyCell::new(
    || {
        [
            (InternalEndpointKind::ConnectionUpgrade,
                EndpointBuilder::default()
                    .id(CONN_UPGRADE_WEBSOCKETS_ENDPOINT_ID.into())
                    .execution_target(ExecutionTarget::Http(
                        HttpTargetBuilder::default()
                            .route("websockets".into())
                            .method(HttpMethod::Get)
                            .version("upgrade".into())
                            .auto_generated(true)
                            .build()
                            .unwrap()
                    ))
                    .description("Handles connection upgrading to WebSockets.".into())
                    .auth(AuthBuilder::default().level(AuthLevel::InjectWhenAvailable).build().unwrap())
                    .build()
                    .unwrap()
            ),
            (
                InternalEndpointKind::Authentication,
                EndpointBuilder::default()
                    .id(LOGIN_ENDPOINT_ID.into())
                    .execution_target(ExecutionTarget::Http(
                        HttpTargetBuilder::default()
                            .route("login".into())
                            .method(HttpMethod::Post)
                            .version("internal".into())
                            .capture_all_params(true)
                            .auto_generated(true)
                            .build()
                            .unwrap()
                    ))
                    .description("Login a user capturing all parameters and forwarding them to the underlying authentication method.".into())
                    .build()
                    .unwrap()
            ),
            (
                InternalEndpointKind::Authentication,
                EndpointBuilder::default()
                    .id(SIGNUP_ENDPOINT_ID.into())
                    .execution_target(ExecutionTarget::Http(
                        HttpTargetBuilder::default()
                            .route("signup".into())
                            .method(HttpMethod::Post)
                            .version("internal".into())
                            .capture_all_params(true)
                            .auto_generated(true)
                            .build()
                            .unwrap()
                    ))
                    .description("Create a new user capturing all parameters and forwarding them to the underlying authentication method.".into())
                    .build()
                    .unwrap()
            ),
            (
                InternalEndpointKind::Authentication,
                EndpointBuilder::default()
                    .id(LOGOUT_ENDPOINT_ID.into())
                    .execution_target(ExecutionTarget::Http(
                        HttpTargetBuilder::default()
                            .route("logout".into())
                            .method(HttpMethod::Get)
                            .version("internal".into())
                            .auto_generated(true)
                            .build()
                            .unwrap()
                    ))
                    .description("Invalidate the current authorization token.".into())
                    .auth(AuthBuilder::default().level(AuthLevel::Required).build().unwrap())
                    .build()
                    .unwrap()
            ),
            (
                InternalEndpointKind::Authentication,
                EndpointBuilder::default()
                    .id(LOGOUT_ALL_ENDPOINT_ID.into())
                    .execution_target(ExecutionTarget::Http(
                        HttpTargetBuilder::default()
                            .route("logout/all".into())
                            .method(HttpMethod::Get)
                            .version("internal".into())
                            .auto_generated(true)
                            .build()
                            .unwrap()
                    ))
                    .description("Invalidate all the authorization tokens of the current user.".into())
                    .auth(AuthBuilder::default().level(AuthLevel::Required).build().unwrap())
                    .build()
                    .unwrap()
            )
        ]
    },
);

/// Specifies the kind of an internal endpoint.
#[derive(PartialEq, Debug)]
pub enum InternalEndpointKind {
    Authentication,
    ConnectionUpgrade,
    Other,
}
