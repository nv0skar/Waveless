// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use databases::*;
use endpoint::*;

/// Establishes a method to discover a given schema and generates endpoints associated to it.
#[typetag::serde]
#[async_trait]
pub trait AnyEndpointGenerator: Any + AnyExt + BoxedAny + DynClone + Send + Sync + Debug {
    async fn generate(&self, db_conns: DbConns) -> Result<(Endpoints, Option<Bytes>)>;

    /// Unique identifier associated with generator's config.
    fn id(&self) -> Result<Bytes>;
}
