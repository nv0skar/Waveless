// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod mysql;

use crate::*;

use databases::*;

/// Trait implemented for every user authentication mechanism.
/// Note that the auth data does not have to live in a SQL database...
#[typetag::serde]
#[async_trait]
pub trait AnyAuthenticationMethod: Any + BoxedAny + DynClone + Send + Sync + Debug {
    fn name(&self) -> &str;
    fn db_id(&self) -> Option<CompactString>;

    /// Check whether the given credentials match for a given user.
    async fn check(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        entries: HashMap<CompactString, CompactString>,
    ) -> Result<Option<UserId>>;

    /// Signup a new user.
    async fn new(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        entries: HashMap<CompactString, CompactString>,
    ) -> Result<CompactString>;

    /// Deletes a user given it's id.
    async fn delete(&self, db_conn: Arc<dyn AnyDatabaseConnection>, user_id: UserId) -> Result<()>;
}

/// Trait implemented for every session's storage backend.
#[typetag::serde]
#[async_trait]
pub trait AnySessionMethod: Any + BoxedAny + DynClone + Send + Sync + Debug {
    fn name(&self) -> &str;
    fn db_id(&self) -> Option<CompactString>;

    fn max_age(&self) -> Option<usize> {
        None
    }

    /// Get whether a user is authenticated by the given session token.
    async fn check(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        token: CompactString,
    ) -> Result<Option<UserId>>;

    /// Create a new session for the given user.
    async fn new(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        user_id: UserId,
    ) -> Result<CompactString>;

    /// Invalidate all session's of the given user.
    async fn invalidate(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        user_id: UserId,
    ) -> Result<()>;

    /// Remove all the expired sessions.
    async fn remove_expired(&self, db_conn: Arc<dyn AnyDatabaseConnection>) -> Result<()>;
}

/// Trait implemented for every role's storage backend.
#[typetag::serde]
#[async_trait]
pub trait AnyRoleMethod: Any + BoxedAny + DynClone + Send + Sync + Debug {
    fn name(&self) -> &str;
    fn db_id(&self) -> Option<CompactString>;

    /// Get the role of the given user.
    async fn get(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        user_id: UserId,
    ) -> Result<Option<CompactString>>;

    /// Set the role of the given user.
    async fn set(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        user_id: UserId,
        role: CompactString,
    ) -> Result<()>;

    /// Remove the role of the given user.
    async fn remove(&self, db_conn: Arc<dyn AnyDatabaseConnection>, user_id: UserId) -> Result<()>;
}
