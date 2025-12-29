// Waveless
// Copyright (C) 2025 Oscar Alvarez Gonzalez

use crate::*;

#[derive(Clone, PartialEq, Serialize, Deserialize, Display, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("Endpoint({}, {}, {:?})", route, method, description)]
#[serde(default)]
pub struct Endpoint {
    route: CompactString,
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    version: Option<CompactString>, // if no version is set the endpoint will be accessible from {api_prefix}/{route}
    method: HttpMethod,
    target_database: DatabaseId,
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    query: Option<String>,
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    description: Option<CompactString>,
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "CheapVec::is_empty")
    )]
    body_params: CheapVec<CompactString>,
    require_auth: bool,
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "CheapVec::is_empty")
    )]
    allowed_roles: CheapVec<CompactString>,
    #[cfg_attr(
        feature = "toml_codec",
        serde(skip_serializing_if = "std::ops::Not::not")
    )]
    auto_generated: bool,
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            route: Default::default(),
            version: Default::default(),
            method: HttpMethod::Post,
            target_database: Default::default(),
            query: Default::default(),
            description: Default::default(),
            body_params: Default::default(),
            require_auth: Default::default(),
            allowed_roles: Default::default(),
            auto_generated: Default::default(),
        }
    }
}
