// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

pub mod mysql;

use crate::*;

use build::*;

use sea_orm::Value; // Switched from sqlx, as sqlx doesn't support conversion into JSON for arbitrary schemas.
use sqlx::{mysql::*, pool::*};

/// The database's connections' pools manager.
/// The primary database won't be in the `ArrayVec` for efficiency.
#[derive(Constructor, Debug)]
pub struct DatabasesConnections {
    primary: (DatabaseId, Arc<dyn AnyDatabaseConnection>),
    inner: ArrayVec<(DatabaseId, Arc<dyn AnyDatabaseConnection>), { DATABASE_LIMIT - 1 }>,
}

// #[derive(Clone, Debug)]
// pub enum AnyDatabaseConnection {
//     MySQL(SqlxMySqlPoolConnection),
// }

#[async_trait]
pub trait AnyDatabaseConnection: Any + DynClone + Send + Sync + Debug {
    fn name(&self) -> &str;

    async fn execute(&self, input: DatabaseInput) -> Result<DatabaseOutput>;
}

#[derive(Debug)]
pub enum DatabaseInput {
    Query(CompactString),
    QueryValues(CompactString, CheapVec<Value, 8>),
    Bytes(Bytes),
    Any(Box<dyn Any + Send + Sync>),
}

#[derive(Debug)]
pub enum DatabaseOutput {
    Bytes(Bytes),
    Any(Box<dyn Any + Send + Sync>),
}

impl DatabasesConnections {
    /// Creates a new databases pools manager and loads it into the `DATABASE_POOL`'s `OnceCell`.
    #[instrument(skip_all)]
    pub async fn load(databases: CheapVec<project::DatabaseConfig>) -> Result<()> {
        if !databases.iter().any(|db| *db.is_primary()) {
            bail!("There is no database set as primary.")
        };

        let mut primary: MaybeUninit<(DatabaseId, Arc<dyn AnyDatabaseConnection>)> =
            MaybeUninit::zeroed();
        let mut inner: ArrayVec<
            (DatabaseId, Arc<dyn AnyDatabaseConnection>),
            { DATABASE_LIMIT - 1 },
        > = ArrayVec::new_const();

        for db_config in databases {
            info!("Creating {}'s pool.", db_config.id());

            let (pool, _) = db_config
                .connection()
                .new_conn(
                    db_config.id().to_owned(),
                    *db_config.pool_min_size(),
                    *db_config.pool_max_size(),
                )
                .await?;

            if *db_config.is_primary() {
                primary.write((db_config.id().to_owned(), pool));
            } else {
                inner.push((db_config.id().to_owned(), pool));
            }
        }

        let database_pools = DatabasesConnections::new(unsafe { primary.assume_init() }, inner);

        DATABASES_CONNS.set(database_pools).unwrap();

        Ok(())
    }

    /// Search for the database given it's id.
    pub fn search(&self, id: Option<DatabaseId>) -> Result<Arc<dyn AnyDatabaseConnection>> {
        if let Some(id) = id {
            if self.primary.0 == id {
                Ok(self.primary.1.to_owned())
            } else {
                self.inner
                    .iter()
                    .find(|(_id, _)| id == _id)
                    .map(|(_, pool)| pool.to_owned())
                    .ok_or(anyhow!("Cannot find a database with the given id."))
            }
        } else {
            Ok(self.primary.1.to_owned())
        }
    }
}

pub async fn check_checksums_in_build(build: Build) -> Result<()> {
    for build_checksum in build.databases_checksums() {
        let db_config = build
            .config()
            .databases()
            .iter()
            .find(|db_config| db_config.id() == build_checksum.database_id())
            .ok_or(anyhow!(
                "There are checksums whose id doesn't match with any database."
            ))?;

        let Some(schema_discovery) = db_config.schema_discovery() else {
            continue;
        };

        let (_, current_checksum) = schema_discovery
            .method()
            .schema(db_config.id().to_owned(), db_config.connection().to_owned())
            .await?;

        if current_checksum != *build_checksum {
            bail!(
                "The database schema has changed since the last build! Build the project again using the current schema."
            );
        } else {
            info!(
                "Database's schema checksum of '{}' has been verified.",
                db_config.id()
            );
        }
    }
    Ok(())
}
