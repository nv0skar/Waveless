// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use build::*;
use databases::*;

use sqlx::{mysql::*, pool::*};

/// TODO: Add docs here.
#[derive(Clone, Debug)]
pub enum AnySchema {
    MySQL(sea_schema::mysql::def::Schema),
}

impl AnySchema {
    pub async fn load_schema(db_config: &project::DatabaseConfig) -> Result<Self> {
        info!("Loading database schema of '{}'.", db_config.id());

        let (db_conn, raw_conn) = AnyDatabaseConnection::new(db_config).await?;

        match db_conn {
            AnyDatabaseConnection::MySQL(_) => {
                let project::DatabaseConnection::MySQL { db, .. } = db_config.connection() else {
                    bail!("Unexpected error, cannot retrieve the database's name.")
                };

                let mysql_raw_pool = raw_conn.downcast::<Pool<MySql>>().unwrap();

                let schema = sea_schema::mysql::discovery::SchemaDiscovery::new(
                    (*mysql_raw_pool).to_owned(),
                    db,
                )
                .discover()
                .await?;

                Ok(Self::MySQL(schema))
            }
            _ => Err(anyhow!(
                "Unimplemented database's schema for connection {:?}!",
                db_conn
            )),
        }
    }

    pub async fn checksum(&self, db_id: CompactString) -> Result<DatabaseChecksum> {
        match self {
            AnySchema::MySQL(schema) => Ok(DatabaseChecksum::new(
                db_id,
                CheapVec::from_slice(
                    &crc32fast::hash(format!("{:?}", schema).as_str().as_bytes()).to_ne_bytes(),
                ),
            )),
            _ => Err(anyhow!("Unimplemented database's schema's checksum!",)),
        }
    }
}
