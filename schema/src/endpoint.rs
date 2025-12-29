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
pub struct Endpoint {
    route: CompactString,
    version: Option<CompactString>, // if no version is set the endpoint will be accessible from {api_prefix}/{route}
    method: HttpMethod,
    query: Option<String>,
    description: Option<CompactString>,
    body_params: CheapVec<CompactString>,
    require_auth: bool,
    allowed_roles: CheapVec<CompactString>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    auto_generated: bool,
}
