// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

use databases::mysql::*;
use project::*;

/// The MySQL discovery strategy will analyze a MySQL database in order to generate a representation of the data model that will be analyzed by the endpoint generator backend.
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("MySQL schema discovery (skipping: {:?})", skip_tables)]
#[getset(get = "pub")]
pub struct MySQLSchemaDiscoveryMethod {
    #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    skip_tables: CheapVec<CompactString, 0>, // Do not forget that auth, session and role tables are also skipped
}

#[typetag::serde(name = "MySQL")]
#[async_trait]
impl AnyDataSchemaDiscoveryMethod for MySQLSchemaDiscoveryMethod {
    async fn schema(
        &self,
        db_conn_config: Arc<dyn AnyDatabaseConnectionConfig>,
    ) -> Result<(Box<dyn Any>, DatabaseChecksum)> {
        let Ok(db_config) = (db_conn_config as Arc<dyn Any + Send + Sync + 'static>)
            .downcast::<MySQLDBConnectionConfig>()
        else {
            bail!("Cannot downcast config back to a MySQL config type.")
        };

        let (_, raw_conn) = db_config
            .new_conn(
                "mysql_discovery_connection".to_compact_string(),
                Some(1),
                Some(1),
            )
            .await?;

        let mysql_raw_pool = raw_conn.downcast::<Pool<MySql>>().unwrap();

        let schema = sea_schema::mysql::discovery::SchemaDiscovery::new(
            (*mysql_raw_pool).to_owned(),
            db_config.db(),
        )
        .discover()
        .await?;

        Ok((
            Box::new(schema.to_owned()),
            DatabaseChecksum::new(
                db_config.db().to_owned(),
                CheapVec::from_slice(
                    &crc32fast::hash(format!("{:?}", schema).as_str().as_bytes()).to_ne_bytes(),
                ),
            ),
        ))
    }
}
