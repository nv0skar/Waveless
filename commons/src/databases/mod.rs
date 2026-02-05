// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use build::*;

use sea_orm::SqlxMySqlPoolConnection; // Switched from sqlx, as sqlx doesn't support conversion into JSON for arbitrary schemas.
use sqlx::{mysql::*, pool::*};

pub static DATABASES_CONNS: OnceCell<DatabasesConnections> = OnceCell::const_new();

/// The database's connections' pools manager.
/// The primary database won't be in the `ArrayVec` for efficiency.
#[derive(Constructor, Debug)]
pub struct DatabasesConnections {
    primary: Arc<(DatabaseId, AnyDatabaseConnection)>,
    inner: Arc<ArrayVec<(DatabaseId, AnyDatabaseConnection), { DATABASE_LIMIT - 1 }>>,
}

#[derive(Clone, Debug)]
pub enum AnyDatabaseConnection {
    MySQL(SqlxMySqlPoolConnection),
}

impl DatabasesConnections {
    /// Creates a new databases pools manager and loads it into the `DATABASE_POOL`'s `OnceCell`.
    #[instrument(skip_all)]
    pub async fn load(databases: CheapVec<project::DatabaseConfig>) -> Result<()> {
        if !databases.iter().any(|db| *db.is_primary()) {
            bail!("There is no database set as primary.")
        };

        let mut primary: MaybeUninit<(DatabaseId, AnyDatabaseConnection)> = MaybeUninit::zeroed();
        let mut inner: ArrayVec<(DatabaseId, AnyDatabaseConnection), { DATABASE_LIMIT - 1 }> =
            ArrayVec::new_const();

        for db_config in databases {
            info!("Creating {}'s pool.", db_config.id());

            let (pool, _) = AnyDatabaseConnection::new(&db_config).await?;

            if *db_config.is_primary() {
                primary.write((db_config.id().to_owned(), pool));
            } else {
                inner.push((db_config.id().to_owned(), pool));
            }
        }

        let database_pools =
            DatabasesConnections::new(Arc::new(unsafe { primary.assume_init() }), Arc::new(inner));

        DATABASES_CONNS.set(database_pools).unwrap();

        Ok(())
    }

    /// Search for the database given it's id.
    pub fn search(&self, id: Option<DatabaseId>) -> Result<&AnyDatabaseConnection> {
        if let Some(id) = id {
            if self.primary.0 == id {
                Ok(&self.primary.1)
            } else {
                self.inner
                    .iter()
                    .find(|(_id, _)| id == _id)
                    .map(|(_, pool)| pool)
                    .ok_or(anyhow!("Cannot find a database with the given id."))
            }
        } else {
            Ok(&self.primary.1)
        }
    }
}

impl AnyDatabaseConnection {
    /// Creates a new database connection from the given config.
    #[instrument(skip_all)]
    pub async fn new(
        db_config: &project::DatabaseConfig,
    ) -> Result<(Self, SmallBox<dyn Any, S64>)> {
        let num_cpus = std::thread::available_parallelism()?.get();

        match db_config.connection() {
            project::DatabaseConnection::MySQL {
                host,
                username,
                password,
                db,
            } => {
                info!(
                    "Creating new MySQL database connection ({}) on {}",
                    host, db
                );

                let conn_options = MySqlConnectOptions::new()
                    .host(&host.ip().to_string())
                    .port(host.port())
                    .username(username)
                    .password(password)
                    .database(db);

                let pool = PoolOptions::<MySql>::new()
                    .min_connections(db_config.pool_min_size().unwrap_or(num_cpus) as u32)
                    .max_connections(db_config.pool_max_size().unwrap_or(num_cpus * 2) as u32)
                    .connect_with(conn_options)
                    .await
                    .map_err(|err| {
                        anyhow!("Failed creating {}'s MySQL pool. {}", db_config.id(), err)
                    })?;

                let pool_wrapper = SqlxMySqlPoolConnection::from(pool.to_owned());

                Ok((Self::MySQL(pool_wrapper), smallbox!(pool)))
            }
            project::DatabaseConnection::ExternalModule { .. } => {
                todo!("External drivers aren't implemented yet!")
            }
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
                "The are checksums whose id doesn't match with any database."
            ))?;

        let schema = schema::AnySchema::load_schema(db_config).await?;

        let current_checksum = schema
            .checksum(build_checksum.database_id().to_owned())
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
