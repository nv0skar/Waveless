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

use waveless_sql::{
    http_execute::{mysql::*, *},
    schema::mysql::*,
};

use sea_schema::mysql::def::Schema;

/// Discovers all endpoints from the project's database and calculate the checksum per database.
/// TODO: Maybe the endpoint generation logic should be delegated to the `AnyDataSchemaDiscoveryMethod` trait.
#[instrument(skip_all)]
pub async fn discover() -> Result<(
    CheapVec<(CompactString, Endpoints), 0>,
    CheapVec<DatabaseChecksum, 0>,
)> {
    let cx = CompilerCx::acquire();

    let project = cx.project();

    let mut db_endpoints = CheapVec::<(CompactString, Endpoints), 0>::new();

    let mut checksums = CheapVec::<DatabaseChecksum, 0>::new();

    for db_config in project.config().databases() {
        // If schema discovery method is not present for the given database id → skip.
        let Some(schema_discovery) = db_config.schema_discovery() else {
            continue;
        };

        // Load the schema.
        let (schema, checksum) = schema_discovery
            .method()
            .schema(db_config.id().to_owned(), db_config.connection().to_owned())
            .await?;

        // Check if checksum for the current db has to be computed.
        if *schema_discovery.checksum() {
            checksums.push(checksum);
        }

        // Discover endpoints from the schema.
        if *schema_discovery.generate_endpoints() {
            // TODO: each discovery method should have it's generic endpoint generation.
            if let Some(mysql_discovery) = schema_discovery
                .method()
                .to_owned()
                .into_arc_any()
                .downcast_ref::<MySQLSchemaDiscoveryMethod>()
            {
                let Ok(mysql_schema) = schema.downcast::<Schema>() else {
                    bail!("Cannot downcast to MySQL schema.")
                };

                let mut discovered_endpoints = Endpoints::new_unchecked(CheapVec::new_const());

                // For each table generate a GET one, GET many, POST, UPDATE and DELETE endpoints.
                for table in mysql_schema.tables {
                    if mysql_discovery
                        .skip_tables()
                        .contains(&table.info.name.to_compact_string())
                    {
                        continue;
                    }

                    // Check whether the table is a view, only the GET many endpoint will be generated.
                    let is_view = table.info.comment.to_lowercase().eq("view");

                    // Get the table primary key. If it is not present only the GET one and POST endpoints will generated.
                    let pk_id = table
                        .columns
                        .iter()
                        .find(|column| column.key == sea_schema::mysql::def::ColumnKey::Primary)
                        .map(|table| table.name.to_owned());

                    if pk_id.is_none() {
                        debug!(
                            "Table {} doesn't have a primary key. Only GET many and POST endpoints will be generated.",
                            table.info.name.to_owned()
                        )
                    }

                    let columns_names = table
                        .columns
                        .iter()
                        .filter(|column| column.key != sea_schema::mysql::def::ColumnKey::Primary)
                        .map(|column| column.name.to_compact_string())
                        .collect::<CheapVec<CompactString>>();

                    let route_one: CompactString =
                        format!("{}/{}", table.info.name.to_lowercase(), "{id}").into();
                    let route_many: CompactString = table.info.name.to_lowercase().into();

                    for method in &[
                        HttpMethod::Get,
                        HttpMethod::Post,
                        HttpMethod::Put,
                        HttpMethod::Delete,
                    ] {
                        match (method, &pk_id) {
                            (HttpMethod::Get, _) => {
                                match &pk_id {
                                    Some(pk_id) if !is_view => {
                                        let mut endpoint_one = EndpointBuilder::default();

                                        endpoint_one
                                            .id(format!("{}_GetOne", table.info.name.to_owned())
                                                .into())
                                            .description(
                                                format!(
                                                    "Get row from {} by it's primary key.",
                                                    table.info.name
                                                )
                                                .into(),
                                            )
                                            .database(db_config.id().to_owned())
                                            .target(Targets::HttpTarget(
                                                HttpTargetBuilder::default()
                                                    .method(*method)
                                                    .version("v1".into())
                                                    .route(route_one.to_owned())
                                                    .execute(Arc::<MySQLExecute>::new(
                                                        SQLQueryWrapper::new(
                                                            format!(
                                                                "SELECT * FROM {} WHERE {} = {}",
                                                                table.info.name, pk_id, "{id}"
                                                            )
                                                            .into(),
                                                        )
                                                        .with_behaviour(SQLBehaviour::Unique)
                                                        .into(),
                                                    ))
                                                    .query_params(CheapVec::new_const())
                                                    .body_params(CheapVec::new_const())
                                                    .capture_all_params(false)
                                                    .auto_generated(true)
                                                    .build()?,
                                            ))
                                            .tags(CheapVec::from_vec(vec![
                                                table.info.name.to_compact_string(),
                                                "get_one".into(),
                                            ]))
                                            .require_auth(false)
                                            .inject_auth_metadata(false)
                                            .allowed_roles(CheapVec::new_const())
                                            .deprecated(false);

                                        discovered_endpoints.add(endpoint_one.build()?)?;
                                    }
                                    _ => (),
                                }

                                let mut endpoint_many = EndpointBuilder::default();

                                endpoint_many
                                    .id(format!("{}_GetMany", table.info.name.to_owned()).into())
                                    .database(db_config.id().to_owned())
                                    .target(Targets::HttpTarget(
                                        HttpTargetBuilder::default()
                                            .method(*method)
                                            .version("v1".into())
                                            .route(route_many.to_owned())
                                            .execute(Arc::<MySQLExecute>::new(
                                                SQLQueryWrapper::new(
                                                    format!("SELECT * FROM {}", table.info.name,)
                                                        .into(),
                                                )
                                                .into(),
                                            ))
                                            .query_params(CheapVec::new_const())
                                            .body_params(CheapVec::new_const())
                                            .capture_all_params(false)
                                            .auto_generated(true)
                                            .build()?,
                                    ))
                                    .description(
                                        format!("Get all rows from {}.", table.info.name).into(),
                                    )
                                    .tags(CheapVec::from_vec(vec![
                                        table.info.name.to_compact_string(),
                                        "get_all".into(),
                                    ]))
                                    .require_auth(false)
                                    .inject_auth_metadata(false)
                                    .allowed_roles(CheapVec::new_const())
                                    .deprecated(false);

                                discovered_endpoints.add(endpoint_many.build()?)?;
                            }
                            (HttpMethod::Post, _) if !is_view => {
                                let mut endpoint = EndpointBuilder::default();

                                endpoint
                                    .id(format!("{}_Post", table.info.name).into())
                                    .database(db_config.id().to_owned())
                                    .target(Targets::HttpTarget(
                                        HttpTargetBuilder::default()
                                            .method(*method)
                                            .version("v1".into())
                                            .route(route_many.to_owned())
                                            .execute(Arc::<MySQLExecute>::new(
                                                SQLQueryWrapper::new(
                                                    format!(
                                                        "INSERT INTO {} ({}) VALUES ({})",
                                                        table.info.name,
                                                        columns_names
                                                            .iter()
                                                            .fold(
                                                                String::new(),
                                                                |last, next| format!(
                                                                    "{}, {}",
                                                                    last, next
                                                                )
                                                            )
                                                            .trim_matches(|c: char| c
                                                                .is_whitespace()
                                                                || c == ','),
                                                        columns_names
                                                            .iter()
                                                            .fold(
                                                                String::new(),
                                                                |last, next| format!(
                                                                    "{}, {{ {} }}",
                                                                    last, next
                                                                )
                                                            )
                                                            .trim_matches(|c: char| c
                                                                .is_whitespace()
                                                                || c == ','),
                                                    )
                                                    .into(),
                                                )
                                                .with_include(false)
                                                .into(),
                                            ))
                                            .query_params(CheapVec::new_const())
                                            .body_params(columns_names.to_owned())
                                            .capture_all_params(false)
                                            .auto_generated(true)
                                            .build()?,
                                    ))
                                    .description(
                                        format!("Insert data into {}.", table.info.name).into(),
                                    )
                                    .tags(CheapVec::from_vec(vec![
                                        table.info.name.to_compact_string(),
                                        "post".into(),
                                    ]))
                                    .require_auth(false)
                                    .inject_auth_metadata(false)
                                    .allowed_roles(CheapVec::new_const())
                                    .deprecated(false);

                                discovered_endpoints.add(endpoint.build()?)?;
                            }
                            (HttpMethod::Put, Some(pk_id)) if !is_view => {
                                let mut endpoint = EndpointBuilder::default();

                                endpoint
                                    .id(format!("{}_Put", table.info.name).into())
                                    .database(db_config.id().to_owned())
                                    .target(Targets::HttpTarget(
                                        HttpTargetBuilder::default()
                                            .method(*method)
                                            .version("v1".into())
                                            .route(route_one.to_owned())
                                            .execute(Arc::<MySQLExecute>::new(
                                                SQLQueryWrapper::new(
                                                    format!(
                                                        "UPDATE {} SET {} WHERE {} = {} ",
                                                        table.info.name,
                                                        columns_names
                                                            .iter()
                                                            .map(|name| format!(
                                                                "{} = {{ {} }}",
                                                                name, name
                                                            ))
                                                            .fold(
                                                                String::new(),
                                                                |last, next| format!(
                                                                    "{}, {}",
                                                                    last, next
                                                                )
                                                            )
                                                            .trim_matches(|c: char| c
                                                                .is_whitespace()
                                                                || c == ','),
                                                        pk_id,
                                                        "{id}"
                                                    )
                                                    .into(),
                                                )
                                                .with_include(false)
                                                .into(),
                                            ))
                                            .query_params(CheapVec::new_const())
                                            .body_params(columns_names.to_owned())
                                            .capture_all_params(false)
                                            .auto_generated(true)
                                            .build()?,
                                    ))
                                    .description(
                                        format!(
                                            "Updates {} on row with the given primary key.",
                                            table.info.name
                                        )
                                        .into(),
                                    )
                                    .tags(CheapVec::from_vec(vec![
                                        table.info.name.to_compact_string(),
                                        "put".into(),
                                    ]))
                                    .require_auth(false)
                                    .inject_auth_metadata(false)
                                    .allowed_roles(CheapVec::new_const())
                                    .deprecated(false);

                                discovered_endpoints.add(endpoint.build()?)?;
                            }
                            (HttpMethod::Delete, Some(pk_id)) if !is_view => {
                                let mut endpoint = EndpointBuilder::default();

                                endpoint
                                    .id(format!("{}_Delete", table.info.name).into())
                                    .database(db_config.id().to_owned())
                                    .target(Targets::HttpTarget(
                                        HttpTargetBuilder::default()
                                            .method(*method)
                                            .version("v1".into())
                                            .route(route_one.to_owned())
                                            .execute(Arc::<MySQLExecute>::new(
                                                SQLQueryWrapper::new(
                                                    format!(
                                                        "DELETE FROM {} WHERE {} = {} ",
                                                        table.info.name, pk_id, "{id}"
                                                    )
                                                    .into(),
                                                )
                                                .with_include(false)
                                                .into(),
                                            ))
                                            .query_params(CheapVec::new_const())
                                            .body_params(CheapVec::new_const())
                                            .capture_all_params(false)
                                            .auto_generated(true)
                                            .build()?,
                                    ))
                                    .description(
                                        format!(
                                            "Deletes data from {} with the given primary key.",
                                            table.info.name
                                        )
                                        .into(),
                                    )
                                    .tags(CheapVec::from_vec(vec![
                                        table.info.name.to_compact_string(),
                                        "delete".into(),
                                    ]))
                                    .require_auth(false)
                                    .inject_auth_metadata(false)
                                    .allowed_roles(CheapVec::new_const())
                                    .deprecated(false);

                                discovered_endpoints.add(endpoint.build()?)?;
                            }
                            _ => {}
                        }
                    }
                }

                db_endpoints.push((db_config.id().to_owned(), discovered_endpoints));
            } else {
                return Err(eyre!(
                    "Unimplemented discovery method or invalid discovery solver for the given database id."
                ));
            }
        }
    }
    Ok((db_endpoints, checksums))
}
