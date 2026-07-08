// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use endpoint::*;
use http_execute::*;

/// TODO: add docs.
#[derive(Getters, MutGetters, Debug)]
#[getset(get = "pub", get_mut = "pub")]
pub struct RequestCx {
    pub request: Request<Incoming>,
    pub method: HttpMethod,
    pub request_params: HashMap<CompactString, ParamValue>,
    pub endpoint: Endpoint,
}
