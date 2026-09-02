// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

/// Trait implemented for every session's storage backend.
#[typetag::serde]
#[async_trait]
pub trait AnySessionMethod: AnyExt + DatabaseConsumer {
    fn max_age(&self) -> Option<usize> {
        None
    }

    /// Get whether a user is authenticated by the given session token.
    async fn check(&self, db_conns: DbConns, token: CompactString) -> Result<Option<UserId>>;

    /// Create a new session for the given user.
    async fn new(&self, db_conns: DbConns, user_id: UserId) -> Result<CompactString>;

    /// Invalidate all session's of the given user.
    async fn invalidate(
        &self,
        db_conns: DbConns,
        user_id: UserId,
        token: Option<CompactString>,
    ) -> Result<()>;

    /// Remove all the expired sessions.
    async fn remove_expired(&self, db_conn: Arc<dyn AnyDatabaseConnection>) -> Result<()>;
}
