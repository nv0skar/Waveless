// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use object::*;

// use sea_orm::Value; // Switched from sqlx, as sqlx doesn't support conversion into JSON for arbitrary schemas.

pub type DbConns = HashMap<DatabaseId, Arc<dyn AnyDatabaseConnection>>;

/// The database's connections' pools manager.
/// The primary database won't be in the `ArrayVec` for efficiency.
#[derive(Clone, Constructor, Debug)]
pub struct DatabasesManager {
    inner: HashMap<DatabaseId, Arc<dyn AnyDatabaseConnection>>,
    primary_id: DatabaseId,
}

impl DatabasesManager {
    pub fn acquire() -> &'static Self {
        DATABASES_CONNS
            .get()
            .ok_or(eyre!("Databases connections should have been initialized."))
            .unwrap()
    }
}

#[async_trait]
pub trait AnyDatabaseConnection: AnyExt {
    async fn execute(&self, input: DatabaseInput) -> Result<DatabaseOutput>;
}

pub trait DatabaseConsumer {
    fn databases(&self) -> CheapVec<DatabaseId>;

    fn get_db_handle(&self) -> Result<DbConns> {
        let db_ids = self.databases();

        let _db_pool = DatabasesManager::acquire();

        Ok(match db_ids.len() {
            0 => HashMap::from([_db_pool.primary_db()?]),
            _ => _db_pool.search_many(&db_ids)?,
        })
    }
}

#[derive(Debug)]
pub enum DatabaseInput {
    Query(CompactString),
    QueryValues(CompactString, CheapVec<CompactString, 8>),
    Bytes(Bytes),
    Any(Box<dyn Any + Send + Sync>),
}

#[derive(Debug)]
pub enum DatabaseOutput {
    Bytes(Bytes),
    Any(Box<dyn Any + Send + Sync>),
}

impl DatabasesManager {
    /// Creates a new databases pools manager and loads it into the `DATABASE_POOL`'s `OnceCell`.
    #[instrument(skip_all)]
    pub async fn load(databases: CheapVec<project::DatabaseConfig>) -> Result<()> {
        if !databases.iter().any(|db| *db.is_primary()) {
            bail!("There is no database set as primary.")
        };

        let mut primary_name: MaybeUninit<CompactString> = MaybeUninit::zeroed();

        let mut inner = HashMap::new();

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

        let database_pools = DatabasesManager::new(inner, unsafe { primary_name.assume_init() });

        DATABASES_CONNS.set(database_pools).unwrap();

        Ok(())
    }

    /// Returns the primary database set for this project if any exists.
    pub fn primary_db(&self) -> Result<(DatabaseId, Arc<dyn AnyDatabaseConnection>)> {
        Ok((
            self.primary_id.to_owned(),
            self.inner
                .get(&self.primary_id)
                .wrap_err("No primary database has been defined for this project.")?
                .to_owned(),
        ))
    }

    /// Searches for the database given it's id.
    pub fn search(&self, id: &DatabaseId) -> Result<Arc<dyn AnyDatabaseConnection>> {
        self.inner
            .get(id)
            .ok_or(eyre!("Cannot find a database with the given id."))
            .map(|val| val.to_owned())
    }

    /// Searches for the databases given their ids.
    pub fn search_many(
        &self,
        ids: &CheapVec<DatabaseId>,
    ) -> Result<HashMap<CompactString, Arc<dyn AnyDatabaseConnection>>> {
        ids.as_ref()
            .iter()
            .map(|id| self.search(id).map(|db_conn| (id.to_owned(), db_conn)))
            .collect::<Result<HashMap<_, _>>>()
    }
}

impl Into<DbConns> for DatabasesManager {
    fn into(self) -> DbConns {
        self.inner
            .iter()
            .map(|(id, conn)| (id.to_owned(), conn.to_owned()))
            .collect::<HashMap<_, _>>()
    }
}

impl ObjectArtifact {
    pub async fn check_checksums_in_object(&self, db_conns: DbConns) -> Result<()> {
        for EndpointGeneratorChecksum { method, checksum } in self.databases_checksums() {
            // Run the endpoint generator.
            let (_, Some(current_checksum)) = method.generate(db_conns.to_owned()).await? else {
                return Err(eyre!("Endpoint generator did not return any checksum.")
                    .suggestion("Are you sure the generator is set to return a checksum?"));
            };

            if current_checksum != *checksum {
                bail!(
                    "The database schema has changed since the last build! Build the project again using the current schema."
                );
            } else {
                info!("Checksum `{}` verified.", hex::encode(checksum));
            }
        }
        Ok(())
    }
}
