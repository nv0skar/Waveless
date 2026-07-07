// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

use sea_orm::{FromQueryResult, QueryResult};

/// TODO: add documentation.
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("SQL queries: {:?}", queries)]
#[getset(get = "pub")]
#[serde(from = "MySQLQueryVariants")]
pub struct MySQLExecute {
    /// If no query is marked to be included in the response the response's body will be empty.
    /// NOTE: queries are executed sequentially.
    queries: CheapVec<MySQLQuery>,
}

boxed_any!(MySQLExecute);

#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("{} (include: {})", query, include)]
#[getset(get = "pub")]
pub struct MySQLQuery {
    query: CompactString,

    /// Whether to include the query result in the response.
    #[serde(default = "default_true", skip_serializing_if = "should_skip")]
    include: bool,

    /// Query behaviour on response.
    #[serde(default, skip_serializing_if = "behaviour_skip")]
    behaviour: MySQLBehaviour,
}

fn behaviour_skip(value: &MySQLBehaviour) -> bool {
    *value == MySQLBehaviour::Permissive
}

fn default_true() -> bool {
    true
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MySQLBehaviour {
    Permissive,

    /// Whether the response from the database is expected not to be empty.
    FailOnEmpty,

    /// Whether the response from the database is expected to be exactly one row (unique) or empty.
    Unique,
}

impl Default for MySQLBehaviour {
    fn default() -> Self {
        Self::Permissive
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum MySQLQueryVariants {
    Structured {
        queries: CheapVec<MySQLQuery>,
    },
    SingleQuery {
        #[serde(flatten)]
        query: MySQLQuery,
    },
}

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
        input: ExecuteRequest,
    ) -> Result<ExecuteResponse, RequestError> {
        let mut queries = self.queries.iter();

        let mut res_buffer = CheapVec::<serde_json::Value>::new();

        while let Some(mysql_query) = queries.next() {
            // Replaces Waveless' client's query's parameters placeholders with MySQL's ones.
            let params_order = mysql_query
                .query()
                .trim_start_matches(|c| c != '{')
                .split('{')
                .map(|sub| sub.split_once('}').unwrap_or_default().0.trim())
                .filter(|sub| !sub.is_empty())
                .collect::<CheapVec<&str>>();

            let mut query = mysql_query
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

            // Replaces Waveless' runtime injected query's parameters placeholders with the value.
            // NOTE: the value will be replaced directly in the MySQL query,
            // be aware that a malformed runtime parameter might cause a SQL
            // injection attack (the attack vector could be in malicious
            // authentications, sessions, roles methods implementations).
            query = query
                .split('|')
                .enumerate()
                .map(|(i, sub)| {
                    if i % 2 != 0 {
                        if let Some(ParamValue::Internal(value)) = input.params.get(sub.trim()) {
                            Ok(value.to_compact_string())
                        } else {
                            Err(RequestError::Expected(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!(
                                    "Expected the runtime parameter `{}`, but it was not injected.",
                                    sub
                                )
                                .into(),
                            ))
                        }
                    } else {
                        Ok(sub.into())
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
                        if let ParamValue::Client(param) = opt {
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
                                    .into(),
                                )
                            })?;

                            query = re.replace_all(&query, "").into();
                        } else {
                            return Err(RequestError::Expected(
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                            format!(
                                                "The endpoint requires '{}', but it wasn't provided in the request.",
                                                param_id
                                            )
                                            .into(),
                                        ));
                        }
                    }
                }
            }

            let res = db_conn
                .execute(DatabaseInput::QueryValues(query, ordered_values))
                .await
                .map_err(|err| {
                    RequestError::Expected(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Query execution error: {}", err).into(),
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
                                .into(),
                        )
                    })?,
                );
            }

            match mysql_query.behaviour() {
                MySQLBehaviour::FailOnEmpty if rows.is_empty() => {
                    Err(RequestError::Expected(
                        StatusCode::BAD_REQUEST,
                        format!("Query result cannot be empty. HINT: error triggered because `FailOnEmpty` is enabled for this `MySQL` execution context, maybe you want to set it to `Permissive`?",).into(),
                    ))?
                }
                MySQLBehaviour::Unique if rows.len() != 1 => Err(RequestError::Expected(
                    StatusCode::BAD_REQUEST,
                    format!("Resource does not exist. HINT: error triggered because `Unique` is enabled for this `MySQL` execution context, maybe you want to set it to `Permissive`?",).into(),
                ))?,
                _ => (),
            }

            if mysql_query.include {
                res_buffer.push(match mysql_query.behaviour() {
                    MySQLBehaviour::Unique => json!(rows.first().unwrap()),
                    _ => json!(&rows),
                });
            }
        }

        match res_buffer.len() {
            0 => Ok(ExecuteResponse::new(None, None)),
            1 => Ok(ExecuteResponse::new(
                None,
                Some(BodyValue::Json(res_buffer.last().unwrap().to_owned())),
            )),
            _ => Ok(ExecuteResponse::new(
                None,
                Some(BodyValue::Json(json!(res_buffer))),
            )),
        }
    }
}

impl From<MySQLQueryVariants> for MySQLExecute {
    fn from(value: MySQLQueryVariants) -> Self {
        match value {
            MySQLQueryVariants::Structured { queries } => Self { queries },
            MySQLQueryVariants::SingleQuery { query: mysql_query } => {
                let queries = mysql_query
                    .query()
                    .split(';')
                    .map(|query| query.into())
                    .filter(|query: &CompactString| !query.is_empty())
                    .map(|query| MySQLQuery {
                        query,
                        include: mysql_query.include,
                        behaviour: mysql_query.behaviour.to_owned(),
                    })
                    .collect::<CheapVec<MySQLQuery>>();

                Self { queries }
            }
        }
    }
}

impl MySQLQueryVariants {
    pub fn new_raw(query: CompactString) -> Self {
        Self::SingleQuery {
            query: MySQLQuery {
                query,
                include: true,
                behaviour: MySQLBehaviour::Permissive,
            },
        }
    }

    pub fn new_raw_with_not_include(query: CompactString) -> Self {
        Self::SingleQuery {
            query: MySQLQuery {
                query,
                include: false,
                behaviour: MySQLBehaviour::Permissive,
            },
        }
    }
}
