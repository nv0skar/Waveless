// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

use sea_orm::{FromQueryResult, QueryResult};

/// TODO: add documentation.
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("SQL query: {:?}", query)]
#[getset(get = "pub")]
pub struct MySQLExecute {
    query: CompactString,
}

boxed_any!(MySQLExecute);

#[typetag::serde(name = "MySQL")]
#[async_trait]
impl AnyExecute for MySQLExecute {
    /// Beware that the params are expected to be `ExecuteParams::StringMap`
    /// and the output will be a `serde_json::Value` that will be
    /// further serialized into JSON.
    async fn execute(
        &self,
        method: HttpMethod,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        input: ExecuteInput,
    ) -> Result<ExecuteOutput, RequestError> {
        // Replaces Waveless' client's query's parameters placeholders with MySQL's ones.
        let params_order = self
            .query()
            .trim_start_matches(|c| c != '{')
            .split('{')
            .map(|sub| sub.split_once('}').unwrap_or_default().0.trim())
            .filter(|sub| !sub.is_empty())
            .collect::<CheapVec<&str>>();

        let mut mysql_query = self
            .query()
            .split('{')
            .map(|sub| {
                if sub.contains('}') {
                    sub.trim_start_matches(|c| c != '}').replace('}', "?")
                } else {
                    sub.to_string()
                }
            })
            .collect::<CompactString>();

        //  Replaces Waveless' runtime injected query's parameters placeholders with the value.
        // NOTE: the value will be replaced directly in the MySQL query,
        // be aware that a malformed runtime parameter might cause a SQL
        // injection attack (the attack vector could be in malicious
        // authentications, sessions, roles methods implementations).
        mysql_query = mysql_query
            .split('|')
            .enumerate()
            .map(|(i, sub)| {
                if i % 2 != 0 {
                    if let Some(ExecuteParamValue::Internal(value)) = input.params.get(sub.trim()) {
                        Ok(value.to_compact_string())
                    } else {
                        Err(RequestError::Expected(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!(
                                "Expected the runtime parameter `{}`, but it was not injected.",
                                sub
                            )
                            .to_compact_string(),
                        ))
                    }
                } else {
                    Ok(sub.to_compact_string())
                }
            })
            .collect::<Result<CompactString, RequestError>>()?;

        // Gets parameter values in the order they appear.
        let mut ordered_values = CheapVec::<_, 8>::new();

        for param_id in params_order.iter() {
            match input
                .params()
                .get(&param_id.to_compact_string())
                .map(|opt| {
                    if let ExecuteParamValue::Client(param) = opt {
                        param.to_owned()
                    } else {
                        None
                    }
                })
                .flatten()
            {
                Some(value) => ordered_values.push(sea_orm::Value::from(value.to_string())),
                None => {
                    if method == HttpMethod::Put {
                        // Modifies the query and strip `?`'s at the positions.
                        // As it is a PUT query we have to strip the column's name, '?' at the current position

                        let re = regex::Regex::new(
                            format!(r#",\s*{}\s*=\s*\?|{}\s*=\s*\?\s*,?"#, param_id, param_id)
                                .as_str(),
                        )
                        .map_err(|err| {
                            RequestError::Expected(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!(
                                    "Cannot create the regex to extract '{}' from the query: {}",
                                    param_id, err
                                )
                                .to_compact_string(),
                            )
                        })?;

                        mysql_query = re.replace_all(&mysql_query, "").to_compact_string();
                    } else {
                        return Err(RequestError::Expected(
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

        let res = db_conn
            .execute(DatabaseInput::QueryValues(mysql_query, ordered_values))
            .await
            .map_err(|err| {
                RequestError::Expected(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Query execution error: {}", err).to_compact_string(),
                )
            })?;

        let DatabaseOutput::Any(res) = res else {
            return Err(RequestError::Other(anyhow!(
                "Unexpected database's executor's output."
            )));
        };

        let res = res.downcast::<Vec<QueryResult>>().map_err(|err| {
            RequestError::Other(anyhow!("Cannot downcast to MySQL query result. {:?}", err))
        })?;

        let mut rows = CheapVec::<_, 0>::new();

        for row in *res {
            rows.push(
                sea_orm::JsonValue::from_query_result(&row, "").map_err(|err| {
                    RequestError::Expected(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Internal error: cannot serialize row into JSON. {}", err)
                            .to_compact_string(),
                    )
                })?,
            );
        }

        return Ok(ExecuteOutput::Json(None, json!(&rows)));
    }
}
