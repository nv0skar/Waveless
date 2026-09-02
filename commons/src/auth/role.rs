// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

/// Trait implemented for every role's storage backend.
#[typetag::serde]
#[async_trait]
pub trait AnyRoleMethod: AnyExt + DatabaseConsumer {
    /// Get the role of the given user.
    async fn get(&self, db_conns: DbConns, user_id: UserId) -> Result<Option<CompactString>>;

    /// Set the role of the given user.
    async fn set(&self, db_conns: DbConns, user_id: UserId, role: CompactString) -> Result<()>;

    /// Remove the role of the given user.
    async fn remove(&self, db_conns: DbConns, user_id: UserId) -> Result<()>;
}
