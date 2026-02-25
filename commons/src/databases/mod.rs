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
    inner: DashMap<DatabaseId, Arc<dyn AnyDatabaseConnection>>,
    primary_name: CompactString,
}

#[async_trait]
pub trait AnyDatabaseConnection: Any + BoxedAny + DynClone + Send + Sync + Debug {
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

        let mut primary_name: MaybeUninit<CompactString> = MaybeUninit::zeroed();

        let inner = DashMap::new();

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
                primary_name.write(db_config.id().to_owned());
            }

            inner.insert(db_config.id().to_owned(), pool);
        }

        let database_pools =
            DatabasesConnections::new(inner, unsafe { primary_name.assume_init() });

        DATABASES_CONNS.set(database_pools).unwrap();

        Ok(())
    }

    /// Search for the database given it's id.
    pub fn search(&self, id: Option<DatabaseId>) -> Result<Arc<dyn AnyDatabaseConnection>> {
        if let Some(id) = id {
            self.inner
                .get(&id)
                .ok_or(anyhow!("Cannot find a database with the given id."))
                .map(|entry| entry.value().to_owned())
        } else {
            Ok(self
                .inner
                .get(&self.primary_name)
                .unwrap()
                .value()
                .to_owned())
        }
    }
}

pub async fn check_checksums_in_build(build: &ExecutorBuild) -> Result<()> {
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
