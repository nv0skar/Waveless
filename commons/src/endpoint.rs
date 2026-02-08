// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use execute::*;

/// Holds all the endpoints, is a wrapper of the `CheapVec<Endpoint>` type.
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
#[serde(default)]
pub struct Endpoints {
    #[serde(
        rename = "endpoints",
        default,
        skip_serializing_if = "should_skip_cheapvec"
    )]
    inner: CheapVec<Endpoint, 0>,
}

impl Endpoints {
    /// Adds a new endpoint. This will check that there is no endpoint with the same method, route and version.
    pub fn add(&mut self, new_endpoint: Endpoint) -> Result<()> {
        let search = self.inner.iter().find(|endpoint| {
            endpoint.method == new_endpoint.method
                && endpoint.route.trim_matches('/') == new_endpoint.route.trim_matches('/')
                && endpoint.version == new_endpoint.version
        });

        match search {
            Some(endpoint) => Err(anyhow!(
                "An equivalent endpoint already exists: you were trying to add '{}', but '{}' is equivalent.",
                new_endpoint,
                endpoint
            )),
            None => {
                self.inner.push(new_endpoint);
                Ok(())
            }
        }
    }

    /// Merges two endpoints buffers
    pub fn merge(&mut self, new_endpoints: Endpoints) -> Result<()> {
        for endpoint in new_endpoints.inner {
            if let Err(err) = self.add(endpoint.to_owned()) {
                warn!(
                    "Cannot add endpoint '{}' to the endpoints buffer. {}",
                    endpoint.id,
                    err.to_string()
                )
            }
        }
        Ok(())
    }
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

/// The main endpoint definition that will be either created by the user or discovered by the compiler.
/// This will be then included in the Waveless project's binary.
#[derive(Clone, Serialize, Deserialize, Constructor, Builder, Getters, Display, Debug)]
#[display("{} -> ({}, {:?}, {:?})", route, method, version, description)]
#[builder(pattern = "mutable")]
#[getset(get = "pub")]
pub struct Endpoint {
    /// Endpoint's unique identifier
    id: CompactString,

    /// Route of the endpoint. Note that this will be prefixed with `{api_prefix}/{version}` (if version is set).
    route: CompactString,

    /// The version of the endpoint, if no version is set the endpoint will be accessible from `{api_prefix}/{route}`.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    version: Option<CompactString>,

    /// Method of the endpoint
    method: HttpMethod,

    /// Sets the database that this endpoint will operate on. If `None` the primary database will be used.
    target_database: Option<DatabaseId>,

    /// Establishes the endpoint handler. Note that if no executor is set, the server will try to handle the request internally.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    execute: Option<Arc<dyn AnyExecute>>,

    /// Sets the endpoint description.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    description: Option<CompactString>,

    /// Sets the tags of this endpoint. By default the target table name will be adde as a tag.
    #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    tags: CheapVec<CompactString, 0>,

    /// DEPRECATED: Path parameters are indicated in the route.
    /// Sets the accepted path parameters.
    // #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    // path_params: CheapVec<CompactString>,

    /// Sets the accepted query parameters.
    #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    query_params: CheapVec<CompactString, 0>,

    /// Sets the accepted body parameters.
    #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    body_params: CheapVec<CompactString, 0>,

    /// Whether to require auth.
    require_auth: bool,

    /// All allowed roles to query the endpoint.
    #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    allowed_roles: CheapVec<CompactString, 0>,

    /// Whether to capture all the request's params.
    /// Useful for internal executors and generic trait implementations.
    #[serde(default, skip_serializing_if = "should_skip")]
    capture_all_params: bool,

    /// Whether this endpoint es deprecated.
    #[serde(default, skip_serializing_if = "should_skip")]
    deprecated: bool,

    /// Whether this endpoint has been automatically generated.
    #[serde(default, skip_serializing_if = "auto_generated_skip")]
    auto_generated: bool,
}

impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            || (self.method == other.method
                && self.route == other.route
                && self.version == other.version)
    }
}

/// Available HTTP methods
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Display, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Unknown,
}

impl From<&str> for HttpMethod {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "get" => HttpMethod::Get,
            "post" => HttpMethod::Post,
            "put" => HttpMethod::Put,
            "delete" => HttpMethod::Delete,
            _ => HttpMethod::Unknown,
        }
    }
}

fn auto_generated_skip(value: &bool) -> bool {
    should_skip(&(!*value))
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            id: "".to_compact_string(),
            route: "".to_compact_string(),
            version: Some("v1".to_compact_string()),
            method: HttpMethod::Get,
            target_database: Default::default(),
            execute: None,
            description: None,
            tags: CheapVec::new(),
            query_params: Default::default(),
            body_params: Default::default(),
            require_auth: false,
            allowed_roles: Default::default(),
            capture_all_params: false,
            deprecated: false,
            auto_generated: false,
        }
    }
}
