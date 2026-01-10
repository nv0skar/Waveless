// Waveless
// Copyright (C) 2025 Oscar Alvarez Gonzalez

use crate::*;

/// Holds all the endpoints, is a wrapper of the CheapVec<Endpoint> type.
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("{:?}", endpoints)]
#[serde(default)]
pub struct Endpoints {
    endpoints: CheapVec<Endpoint>,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            endpoints: Default::default(),
        }
    }
}

/// The main endpoint definition that will be either created by the user or discovered by the compiler.
/// This will be then included in the Waveless project's binary.
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("Endpoint({}, {}, {:?})", route, method, description)]
#[serde(default)]
pub struct Endpoint {
    /// Route of the endpoint. Note that this will be prefixed with `{api_prefix}/{version}` (if version is set).
    route: CompactString,

    /// The version of the endpoint, if no version is set the endpoint will be accessible from `{api_prefix}/{route}`.
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    version: Option<CompactString>,

    /// Method of the endpoint
    method: HttpMethod,

    /// Sets the database that this endpoint will operate on. If `None` the main database will be used.
    target_database: Option<DatabaseId>,

    /// Establishes the endpoint handler. Note that if no executor is set, the server will try to handle the request internally.
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    executor: Option<Executor>,

    /// Sets the endpoint description.
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    description: Option<CompactString>,

    /// Sets the tags of this endpoint. By default the target table name will be adde as a tag.
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "CheapVec::is_empty")
    )]
    tags: CheapVec<CompactString>,

    /// Sets the accepted path parameters.
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "CheapVec::is_empty")
    )]
    path_params: CheapVec<CompactString>,

    /// Sets the accepted query parameters.
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "CheapVec::is_empty")
    )]
    query_params: CheapVec<CompactString>,

    /// Sets the accepted body parameters.
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "CheapVec::is_empty")
    )]
    body_params: CheapVec<CompactString>,

    /// Whether to require auth.
    require_auth: bool,

    /// All allowed roles to query the endpoint.
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "CheapVec::is_empty")
    )]
    allowed_roles: CheapVec<CompactString>,

    /// Whether this endpoint es deprecated.
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "deprecated_skip"))]
    deprecated: bool,

    /// Whether this endpoint has been automatically generated.
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "std::ops::Not::not")
    )]
    auto_generated: bool,
}

/// Available HTTP methods
#[derive(Clone, PartialEq, Serialize, Deserialize, Display, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

fn deprecated_skip(value: &bool) -> bool {
    *value
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            route: "/products/".to_compact_string(),
            version: Some("v1".to_compact_string()),
            method: HttpMethod::Get,
            target_database: Default::default(),
            executor: Some(Executor::SQL {
                query: "SELECT * FROM products WHERE size = {size}".to_compact_string(),
            }),
            description: Some("Get all the products by the given size.".to_compact_string()),
            tags: CheapVec::from_vec(vec!["products".to_compact_string()]),
            path_params: CheapVec::from_vec(vec!["size".to_compact_string()]),
            query_params: Default::default(),
            body_params: Default::default(),
            require_auth: false,
            allowed_roles: Default::default(),
            deprecated: false,
            auto_generated: false,
        }
    }
}

/// Defines all methods available to handle requests to the endpoints.
#[derive(Clone, PartialEq, Serialize, Deserialize, Display, Debug)]
// #[cfg_attr(feature = "toml_codec", serde(tag = "type"))]
pub enum Executor {
    #[display("SQL query: {:?}", query)]
    SQL { query: CompactString },

    #[display("Hook name: {:?}", fn_name)]
    Hook { fn_name: CompactString },
}
