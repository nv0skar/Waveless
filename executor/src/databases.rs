// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// The database's connections' pools manager.
/// The primary database won't be in the `DashMap` for efficiency.
#[derive(Constructor, Debug)]
pub struct DatabasesConnections {
    primary: Arc<(DatabaseId, AnyDatabaseConnection)>,
    inner: Arc<ArrayVec<(DatabaseId, AnyDatabaseConnection), { DATABASE_LIMIT - 1 }>>,
}

#[derive(Debug)]
pub enum AnyDatabaseConnection {
    MySQL(SqlxMySqlPoolConnection),
}

impl DatabasesConnections {
    /// Creates a new database pools manager and loads it into the `DATABASE_POOL`'s `OnceCell`.
    pub async fn load() -> Result<()> {
        let build = build_loader::build()?;
        let databases = build.general().databases();

        if !databases.iter().any(|db| *db.is_primary()) {
            bail!("There is no database set as primary.")
        };

        let mut primary: MaybeUninit<(DatabaseId, AnyDatabaseConnection)> = MaybeUninit::zeroed();
        let mut inner: ArrayVec<(DatabaseId, AnyDatabaseConnection), { DATABASE_LIMIT - 1 }> =
            ArrayVec::new_const();

        for database in databases {
            debug!("Creating {}'s pool.", database.id());

            let num_cpus = std::thread::available_parallelism()?.get();

            let pool = match database.connection() {
                project::DatabaseConnection::MySQL {
                    host,
                    username,
                    password,
                    db,
                } => {
                    let conn_options = MySqlConnectOptions::new()
                        .host(&host.ip().to_string())
                        .port(host.port())
                        .username(username)
                        .password(password)
                        .database(db);

                    let pool = PoolOptions::<MySql>::new()
                        .min_connections(database.pool_min_size().unwrap_or(num_cpus) as u32)
                        .max_connections(database.pool_max_size().unwrap_or(num_cpus * 2) as u32)
                        .connect_with(conn_options)
                        .await
                        .map_err(|err| {
                            anyhow!("Failed creating {}'s MySQL pool. {}", database.id(), err)
                        })?;

                    let pool_wrapper = SqlxMySqlPoolConnection::from(pool);

                    AnyDatabaseConnection::MySQL(pool_wrapper)
                }
                project::DatabaseConnection::ExternalModule { .. } => {
                    todo!("External drivers aren't implemented yet!")
                }
            };

            if *database.is_primary() {
                primary.write((database.id().to_owned(), pool));
            } else {
                inner.push((database.id().to_owned(), pool));
            }
        }

        let database_pools =
            DatabasesConnections::new(Arc::new(unsafe { primary.assume_init() }), Arc::new(inner));

        DATABASE_POOLS.set(database_pools).unwrap();

        Ok(())
    }

    /// Search for the given database id in the pool.
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
