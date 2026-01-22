// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

#[derive(Clone, PartialEq, Constructor, Debug)]
pub struct ExecuteExt(endpoint::Execute);

impl ExecuteExt {
    /// Executes a query using the given executor and database connection.
    /// Beware that the output must be a `serde_json::Value` that will be
    /// further serialized into JSON
    #[instrument(skip_all)]
    pub async fn execute(
        &self,
        method: endpoint::HttpMethod,
        db_conn: &AnyDatabaseConnection,
        params: HashMap<CompactString, Option<CompactString>>,
    ) -> Result<serde_json::Value, ConnHandlerError> {
        match &self.0 {
            endpoint::Execute::MySQL { query } => {
                let AnyDatabaseConnection::MySQL(mysql_pool) = db_conn;

                // Replaces Waveless' query's parameters placeholders with MySQL's ones.
                let params_order = query
                    .trim_start_matches(|c| c != '{')
                    .split('{')
                    .map(|sub| sub.split_once('}').unwrap_or_default().0.trim())
                    .filter(|sub| !sub.is_empty())
                    .collect::<CheapVec<&str>>();

                let mut mysql_query = query
                    .split('{')
                    .map(|sub| {
                        if sub.contains('}') {
                            sub.trim_start_matches(|c| c != '}').replace('}', "?")
                        } else {
                            sub.to_string()
                        }
                    })
                    .collect::<CompactString>();

                // Gets parameter values in the order they appear.
                let mut ordered_values = CheapVec::new();

                for param_id in params_order.iter() {
                    match params
                        .get(&param_id.to_compact_string())
                        .map(|opt| opt.to_owned())
                        .flatten()
                    {
                        Some(value) => ordered_values.push(sea_orm::Value::from(value.to_string())),
                        None => {
                            if method == endpoint::HttpMethod::Put {
                                // Modifies the query and strip `?`'s at the positions.
                                // As it is a PUT query we have to strip the column's name, '?' at the current position

                                let re = regex::Regex::new(
                                    format!(r#",\s*{}\s*=\s*\?|{}\s*=\s*\?\s*,?"#, param_id, param_id).as_str(),
                                )
                                .map_err(|err| {
                                    ConnHandlerError::Expected(
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        format!("Cannot create the regex to extract '{}' from the query: {}", param_id, err)
                                            .to_compact_string(),
                                    )
                                })?;

                                mysql_query = re.replace_all(&mysql_query, "").to_compact_string();
                            } else {
                                return Err(ConnHandlerError::Expected(
                           StatusCode::INTERNAL_SERVER_ERROR,
                            format!(
                                "The endpoint requires '{}', but it wasn't provided in the request.",
                                param_id
                            )
                            .to_compact_string(),
                        ));
                            }
                        }
                    }
                }

                let res = mysql_pool
                    .query_all(Statement::from_sql_and_values(
                        DbBackend::MySql,
                        mysql_query.to_string(),
                        ordered_values,
                    ))
                    .await
                    .map_err(|err| {
                        ConnHandlerError::Expected(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Query execution error: {}", err).to_compact_string(),
                        )
                    })?;

                let mut rows = CheapVec::new();

                for row in res {
                    rows.push(
                        sea_orm::JsonValue::from_query_result(&row, "").map_err(|err| {
                            ConnHandlerError::Expected(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Internal error: cannot serialize row into JSON. {}", err)
                                    .to_compact_string(),
                            )
                        })?,
                    );
                }

                return Ok(json!(&rows));
            }
            endpoint::Execute::Hook { .. } => {
                todo!("Custom endpoint hooks aren't implemented yet.")
            }
        }
    }
}
