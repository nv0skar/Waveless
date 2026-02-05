// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

//!
//! The Waveless' endpoints discovery and schema's checksum calculator.
//! Connects to the specified the project's databases, scans their schema to produce
//! endpoints accordingly and produces the schema's checksum.
//! NOTE: Instead of making schema discovery and database's schema's checksum calculation different methods
//! they have been unified into a single method, which in turn opens only one connection for databases
//! that require both endpoint discovery and schema's checksum, also this avoid complex global connection
//! handling per database.
//!
use crate::*;

/// Discovers all endpoints from the project's database and calculate the checksum per database.
#[instrument(skip_all)]
pub async fn discover() -> Result<(
    CheapVec<(CompactString, Endpoints), 0>,
    CheapVec<DatabaseChecksum, 0>,
)> {
    let config = config_loader::project_config()?;

    let mut db_endpoints = CheapVec::<(CompactString, Endpoints), 0>::new();

    let mut checksums = CheapVec::<DatabaseChecksum, 0>::new();

    for db_config in config.config().databases() {
        let discovery_config =
            config
                .compiler()
                .endpoint_discovery()
                .iter()
                .find(|discovery_config| {
                    if let Some(db_id) = discovery_config.database_id() {
                        db_config.id() == db_id
                    } else {
                        *db_config.is_primary()
                    }
                });

        // If schema discovery and checksum is disabled → skip.
        if discovery_config.is_none() && !db_config.checksum_schema() {
            continue;
        }

        // Load the schema.
        let schema = AnySchema::load_schema(db_config).await?;

        // Check if checksum for the current db has to be computed.
        if *db_config.checksum_schema() {
            checksums.push(schema.checksum(db_config.id().to_owned()).await?);
        }

        // Discover endpoints from the schema.
        if let Some(discovery_config) = discovery_config {
            match (discovery_config.method(), schema.to_owned()) {
                (
                    project::DataSchemaDiscoveryMethod::MySQL { skip_tables },
                    AnySchema::MySQL(mysql_schema),
                ) => {
                    let mut discovered_endpoints = Endpoints::new(CheapVec::new());

                    // For each table generate a GET, POST, UPDATE and DELETE endpoints.
                    for table in mysql_schema.tables {
                        if skip_tables.contains(&table.info.name.to_compact_string()) {
                            continue;
                        }

                        let pk_id = table
                            .columns
                            .iter()
                            .find(|column| column.key == sea_schema::mysql::def::ColumnKey::Primary)
                            .ok_or(anyhow!(
                                "Table {} doesn't have a primary key.",
                                table.info.name
                            ))?
                            .to_owned()
                            .name;

                        let columns_names = table
                            .columns
                            .iter()
                            .filter(|column| {
                                column.key != sea_schema::mysql::def::ColumnKey::Primary
                            })
                            .map(|column| column.name.to_compact_string())
                            .collect::<CheapVec<CompactString>>();

                        let route_one = format!("{}/{}", table.info.name.to_lowercase(), "{id}")
                            .to_compact_string();
                        let route_many = table.info.name.to_lowercase().to_compact_string();

                        const METHODS_TO_GENERATE: &[HttpMethod] = &[
                            HttpMethod::Get,
                            HttpMethod::Post,
                            HttpMethod::Put,
                            HttpMethod::Delete,
                        ];

                        for method in METHODS_TO_GENERATE {
                            match method {
                                HttpMethod::Get => {
                                    let mut endpoint_one = EndpointBuilder::default();
                                    let mut endpoint_many = EndpointBuilder::default();

                                    endpoint_one
                                        .id(format!("{}_GetOne", table.info.name)
                                            .to_compact_string())
                                        .method(*method)
                                        .version(Some("v1".to_compact_string()))
                                        .route(route_one.to_owned())
                                        .description(Some(
                                            format!(
                                                "Get row from {} by it's primary key.",
                                                table.info.name
                                            )
                                            .to_compact_string(),
                                        ))
                                        .target_database(Some(db_config.id().to_owned()))
                                        .execute(Some(Execute::MySQL {
                                            query: format!(
                                                "SELECT * FROM {} WHERE {} = {}",
                                                table.info.name, pk_id, "{id}"
                                            )
                                            .to_compact_string(),
                                        }))
                                        .tags(CheapVec::from_vec(vec![
                                            table.info.name.to_compact_string(),
                                            "get_one".to_compact_string(),
                                        ]))
                                        .query_params(CheapVec::new())
                                        .body_params(CheapVec::new())
                                        .require_auth(false)
                                        .allowed_roles(CheapVec::new())
                                        .deprecated(false)
                                        .auto_generated(true);

                                    endpoint_many
                                        .id(format!("{}_GetMany", table.info.name)
                                            .to_compact_string())
                                        .method(*method)
                                        .version(Some("v1".to_compact_string()))
                                        .route(route_many.to_owned())
                                        .description(Some(
                                            format!("Get all rows from {}.", table.info.name)
                                                .to_compact_string(),
                                        ))
                                        .target_database(Some(db_config.id().to_owned()))
                                        .execute(Some(Execute::MySQL {
                                            query: format!("SELECT * FROM {}", table.info.name,)
                                                .to_compact_string(),
                                        }))
                                        .tags(CheapVec::from_vec(vec![
                                            table.info.name.to_compact_string(),
                                            "get_all".to_compact_string(),
                                        ]))
                                        .query_params(CheapVec::new())
                                        .body_params(CheapVec::new())
                                        .require_auth(false)
                                        .allowed_roles(CheapVec::new())
                                        .deprecated(false)
                                        .auto_generated(true);

                                    discovered_endpoints.add(endpoint_one.build()?)?;
                                    discovered_endpoints.add(endpoint_many.build()?)?;
                                }
                                HttpMethod::Post => {
                                    let mut endpoint = EndpointBuilder::default();

                                    endpoint
                                        .id(format!("{}_Post", table.info.name).to_compact_string())
                                        .method(*method)
                                        .version(Some("v1".to_compact_string()))
                                        .route(route_many.to_owned())
                                        .description(Some(
                                            format!("Insert data into {}.", table.info.name)
                                                .to_compact_string(),
                                        ))
                                        .target_database(Some(db_config.id().to_owned()))
                                        .execute(Some(Execute::MySQL {
                                            query: format!(
                                                "INSERT INTO {} ({}) VALUES ({})",
                                                table.info.name,
                                                columns_names
                                                    .iter()
                                                    .fold(String::new(), |last, next| format!(
                                                        "{}, {}",
                                                        last, next
                                                    ))
                                                    .trim_matches(
                                                        |c: char| c.is_whitespace() || c == ','
                                                    ),
                                                columns_names
                                                    .iter()
                                                    .fold(String::new(), |last, next| format!(
                                                        "{}, {{ {} }}",
                                                        last, next
                                                    ))
                                                    .trim_matches(
                                                        |c: char| c.is_whitespace() || c == ','
                                                    ),
                                            )
                                            .to_compact_string(),
                                        }))
                                        .body_params(columns_names.to_owned())
                                        .tags(CheapVec::from_vec(vec![
                                            table.info.name.to_compact_string(),
                                            "post".to_compact_string(),
                                        ]))
                                        .query_params(CheapVec::new())
                                        .body_params(columns_names.to_owned())
                                        .require_auth(false)
                                        .allowed_roles(CheapVec::new())
                                        .deprecated(false)
                                        .auto_generated(true);

                                    discovered_endpoints.add(endpoint.build()?)?;
                                }
                                HttpMethod::Put => {
                                    let mut endpoint = EndpointBuilder::default();

                                    endpoint
                                        .id(format!("{}_Put", table.info.name).to_compact_string())
                                        .method(*method)
                                        .version(Some("v1".to_compact_string()))
                                        .route(route_one.to_owned())
                                        .description(Some(
                                            format!(
                                                "Updates {} on row with the given primary key.",
                                                table.info.name
                                            )
                                            .to_compact_string(),
                                        ))
                                        .target_database(Some(db_config.id().to_owned()))
                                        .execute(Some(Execute::MySQL {
                                            query: format!(
                                                "UPDATE {} SET {} WHERE {} = {} ",
                                                table.info.name,
                                                columns_names
                                                    .iter()
                                                    .map(|name| format!(
                                                        "{} = {{ {} }}",
                                                        name, name
                                                    ))
                                                    .fold(String::new(), |last, next| format!(
                                                        "{}, {}",
                                                        last, next
                                                    ))
                                                    .trim_matches(
                                                        |c: char| c.is_whitespace() || c == ','
                                                    ),
                                                pk_id,
                                                "{id}"
                                            )
                                            .to_compact_string(),
                                        }))
                                        .tags(CheapVec::from_vec(vec![
                                            table.info.name.to_compact_string(),
                                            "put".to_compact_string(),
                                        ]))
                                        .query_params(CheapVec::new())
                                        .body_params(columns_names.to_owned())
                                        .require_auth(false)
                                        .allowed_roles(CheapVec::new())
                                        .deprecated(false)
                                        .auto_generated(true);

                                    discovered_endpoints.add(endpoint.build()?)?;
                                }
                                HttpMethod::Delete => {
                                    let mut endpoint = EndpointBuilder::default();

                                    endpoint
                                        .id(format!("{}_Delete", table.info.name)
                                            .to_compact_string())
                                        .method(*method)
                                        .version(Some("v1".to_compact_string()))
                                        .route(route_one.to_owned())
                                        .description(Some(
                                            format!(
                                                "Deletes data from {} with the given primary key.",
                                                table.info.name
                                            )
                                            .to_compact_string(),
                                        ))
                                        .target_database(Some(db_config.id().to_owned()))
                                        .execute(Some(Execute::MySQL {
                                            query: format!(
                                                "DELETE FROM {} WHERE {} = {} ",
                                                table.info.name, pk_id, "{id}"
                                            )
                                            .to_compact_string(),
                                        }))
                                        .body_params(columns_names.to_owned())
                                        .tags(CheapVec::from_vec(vec![
                                            table.info.name.to_compact_string(),
                                            "delete".to_compact_string(),
                                        ]))
                                        .query_params(CheapVec::new())
                                        .body_params(CheapVec::new())
                                        .require_auth(false)
                                        .allowed_roles(CheapVec::new())
                                        .deprecated(false)
                                        .auto_generated(true);

                                    discovered_endpoints.add(endpoint.build()?)?;
                                }
                                HttpMethod::Unknown => {}
                            }
                        }
                    }

                    db_endpoints.push((db_config.id().to_owned(), discovered_endpoints));
                }
                _ => {
                    return Err(anyhow!(
                        "Unimplemented discovery method or invalid discovery solver for the given database id."
                    ));
                }
            }
        }
    }
    Ok((db_endpoints, checksums))
}
