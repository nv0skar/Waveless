// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

use project::*;

use sea_orm::SqlxMySqlPoolConnection;
use sea_orm::{DbBackend, Statement};

#[derive(Clone, Debug)]
pub struct MySQLConnection(SqlxMySqlPoolConnection);

boxed_any!(MySQLConnection);

#[async_trait]
impl AnyDatabaseConnection for MySQLConnection {
    fn name(&self) -> &str {
        "mysql"
    }

    async fn execute(&self, input: DatabaseInput) -> Result<DatabaseOutput> {
        match input {
            DatabaseInput::Query(query) => {
                let res = self
                    .0
                    .query_all(Statement::from_string(DbBackend::MySql, query.to_string()))
                    .await?;

                Ok(DatabaseOutput::Any(Box::new(res)))
            }
            DatabaseInput::QueryValues(query, params) => {
                let res = self
                    .0
                    .query_all(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        query.to_string(),
                        params,
                    ))
                    .await?;

                Ok(DatabaseOutput::Any(Box::new(res)))
            }
            _ => Err(anyhow!("Unsupported input for MySQL query.")),
        }
    }
}

/// MySQL database
// TODO - Support more authentication methods
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("MySQL: {}@{} on {}", username, host, db)]
#[getset(get = "pub")]
pub struct MySQLDBConnectionConfig {
    host: SocketAddr,
    username: CompactString,
    password: CompactString,
    db: CompactString,
}

boxed_any!(MySQLDBConnectionConfig);

#[typetag::serde(name = "MySQL")]
#[async_trait]
impl AnyDatabaseConnectionConfig for MySQLDBConnectionConfig {
    async fn new_conn(
        &self,
        id: CompactString,
        pool_min_size: Option<usize>,
        pool_max_size: Option<usize>,
    ) -> Result<(Arc<dyn AnyDatabaseConnection>, Box<dyn Any>)> {
        info!(
            "Creating new MySQL database connection ({}) on {}",
            self.host, self.db
        );

        let num_cpus = std::thread::available_parallelism()?.get();

        let conn_options = MySqlConnectOptions::new()
            .host(&self.host.ip().to_string())
            .port(self.host.port())
            .username(&self.username)
            .password(&self.password)
            .database(&self.db);

        let pool = PoolOptions::<MySql>::new()
            .min_connections(pool_min_size.unwrap_or(num_cpus) as u32)
            .max_connections(pool_max_size.unwrap_or(num_cpus * 2) as u32)
            .connect_with(conn_options)
            .await
            .map_err(|err| anyhow!("Failed creating {}'s MySQL pool. {}", id, err))?;

        let pool_wrapper = SqlxMySqlPoolConnection::from(pool.to_owned());

        Ok((Arc::new(MySQLConnection(pool_wrapper)), Box::new(pool)))
    }
}
