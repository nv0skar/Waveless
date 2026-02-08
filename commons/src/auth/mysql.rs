// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

use databases::mysql::*;

use sea_orm::QueryResult;

#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("Name & password authentication on SQL using table {}", table_name)]
#[getset(get = "pub")]
pub struct MySQLSimpleAuthenticationMethod {
    /// Will use the primary database by default.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    database_id: Option<DatabaseId>,
    table_name: CompactString,
    user_field: CompactString,
    /// This field references to the user table in order to model a relationship and implement login with name, emails, IDs... Must not be primary key.
    name_field: CompactString,
    password_field: CompactString,
    totp_field: Option<CompactString>,
}

boxed_any!(MySQLSimpleAuthenticationMethod);

impl Default for MySQLSimpleAuthenticationMethod {
    fn default() -> Self {
        Self {
            database_id: None,
            table_name: "users_auth".to_compact_string(),
            user_field: "user_id".to_compact_string(),
            name_field: "email".to_compact_string(),
            password_field: "password".to_compact_string(),
            totp_field: None,
        }
    }
}

#[derive(Clone, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("SQL backed token on table {}", table_name)]
#[getset(get = "pub")]
pub struct MySQLToken {
    /// Will use the primary database by default.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    database_id: Option<DatabaseId>,
    table_name: CompactString,
    token_field: CompactString,
    /// Must not be primary key.
    user_field: CompactString,
    created_field: CompactString,
    /// Max age of sessions.
    max_age: usize,
}

boxed_any!(MySQLToken);

impl Default for MySQLToken {
    fn default() -> Self {
        Self {
            database_id: None,
            table_name: "sessions_auth".to_compact_string(),
            token_field: "session_id".to_compact_string(),
            user_field: "user_id".to_compact_string(),
            created_field: "created_at".to_compact_string(),
            max_age: 86400,
        }
    }
}

#[derive(Clone, Constructor, Serialize, Deserialize, Getters, Display, Debug)]
#[display("SQL backed users' roles check on {}", table_name)]
#[getset(get = "pub")]
pub struct MySQLRole {
    /// Will use the primary database by default.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    database_id: Option<DatabaseId>,
    table_name: CompactString,
    user_field: CompactString,
    /// Must not be primary key.
    role_field: CompactString,
}

boxed_any!(MySQLRole);

impl Default for MySQLRole {
    fn default() -> Self {
        Self {
            database_id: None,
            table_name: "roles_auth".to_compact_string(),
            user_field: "user_id".to_compact_string(),
            role_field: "role".to_compact_string(),
        }
    }
}

#[typetag::serde(name = "MySQLSimple")]
#[async_trait]
impl AnyAuthenticationMethod for MySQLSimpleAuthenticationMethod {
    fn name(&self) -> &'static str {
        "mysqlsimple"
    }

    fn db_id(&self) -> Option<CompactString> {
        self.database_id.to_owned()
    }

    async fn check(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        entries: HashMap<CompactString, CompactString>,
    ) -> Result<Option<UserId>> {
        let Ok(db_conn) = db_conn
            .to_owned()
            .into_arc_any()
            .downcast::<MySQLConnection>()
        else {
            bail!(
                "Database connection for `MySQLSimple` authentication should be of type {:?} but it's of type {:?}.",
                TypeId::of::<MySQLDBConnectionConfig>(),
                db_conn.inner_type_id()
            )
        };

        let name_field = entries
            .get(&self.name_field)
            .ok_or(anyhow!("'{}' field not found.", self.name_field))?;
        let password_field = entries
            .get(&self.password_field)
            .ok_or(anyhow!("'{}' field not found.", self.password_field))?;

        let res = db_conn
            .execute(DatabaseInput::QueryValues(
                format!(
                    "SELECT {} FROM {} WHERE {} = ? AND {} = ?",
                    self.user_field, self.table_name, self.name_field, self.password_field
                )
                .to_compact_string(),
                CheapVec::from_vec(vec![
                    sea_orm::Value::from(name_field.to_string()),
                    sea_orm::Value::from(password_field.to_string()),
                ]),
            ))
            .await
            .map_err(|err| anyhow!("Query execution error: {}", err))?;

        let DatabaseOutput::Any(res) = res else {
            bail!("Unexpected database's executor's output.");
        };

        let res = res.downcast::<Vec<QueryResult>>().map_err(|err| {
            RequestError::Other(anyhow!("Cannot downcast to MySQL query result. {:?}", err))
        })?;

        let Some(entry) = res.first() else {
            return Ok(None);
        };

        let Ok(user_id) = entry.try_get::<u32>("", &self.user_field) else {
            bail!(
                "Field '{}' expected but not returned in '{}' table. Maybe it exists but the associated data type is not `INT UNSIGNED`.",
                self.user_field,
                self.table_name
            )
        };

        Ok(Some(user_id as usize))
    }

    async fn new(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        entries: HashMap<CompactString, CompactString>,
    ) -> Result<CompactString> {
        todo!()
    }

    async fn delete(&self, db_conn: Arc<dyn AnyDatabaseConnection>, user_id: UserId) -> Result<()> {
        todo!()
    }
}

#[typetag::serde(name = "MySQLToken")]
#[async_trait]
impl AnySessionMethod for MySQLToken {
    fn name(&self) -> &'static str {
        "mysqltoken"
    }

    fn db_id(&self) -> Option<CompactString> {
        self.database_id.to_owned()
    }

    async fn check(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        token: CompactString,
    ) -> Result<Option<UserId>> {
        let Ok(db_conn) = db_conn
            .to_owned()
            .into_arc_any()
            .downcast::<MySQLConnection>()
        else {
            bail!(
                "Database connection for `MySQLToken` session method should be of type {:?} but it's of type {:?}.",
                TypeId::of::<MySQLDBConnectionConfig>(),
                db_conn.inner_type_id()
            )
        };

        let res = db_conn
            .execute(DatabaseInput::QueryValues(
                format!(
                    "SELECT {}, {} FROM {} WHERE {} = ?",
                    self.user_field, self.created_field, self.table_name, self.token_field
                )
                .to_compact_string(),
                CheapVec::from_vec(vec![sea_orm::Value::from(token.to_string())]),
            ))
            .await
            .map_err(|err| anyhow!("Query execution error: {}", err))?;

        let DatabaseOutput::Any(res) = res else {
            bail!("Unexpected database's executor's output.");
        };

        let res = res.downcast::<Vec<QueryResult>>().map_err(|err| {
            RequestError::Other(anyhow!("Cannot downcast to MySQL query result. {:?}", err))
        })?;

        let Some(entry) = res.first() else {
            return Ok(None);
        };

        let Ok(user_id) = entry.try_get::<u32>("", &self.user_field) else {
            bail!(
                "Field '{}' expected but not returned in '{}' table. Maybe it exists but the associated data type is not `INT UNSIGNED`.",
                self.user_field,
                self.table_name
            )
        };

        let Ok(created_at) = entry.try_get::<NaiveDateTime>("", &self.created_field) else {
            bail!(
                "Cannot find field '{}' in '{}' table. Maybe it exists but the associated data type is not `DATETIME`.",
                self.created_field,
                self.table_name
            )
        };

        // Checks whether the token has expired.
        if created_at + Duration::from_secs(self.max_age as u64) <= Utc::now().naive_utc()
            || created_at > Utc::now().naive_utc()
        {
            // TODO: remove the expired or invalid token.
            return Ok(None);
        }

        Ok(Some(user_id as usize))
    }

    async fn new(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        user_id: UserId,
    ) -> Result<CompactString> {
        let Ok(db_conn) = db_conn
            .to_owned()
            .into_arc_any()
            .downcast::<MySQLConnection>()
        else {
            bail!(
                "Database connection for `MySQLToken` session method should be of type {:?} but it's of type {:?}.",
                TypeId::of::<MySQLDBConnectionConfig>(),
                db_conn.inner_type_id()
            )
        };

        let token = Alphanumeric
            .sample_string(&mut rand::rng(), 32)
            .to_compact_string();

        let _ = db_conn
            .execute(DatabaseInput::QueryValues(
                format!("INSERT INTO {} VALUES (?, ?, ?)", self.table_name).to_compact_string(),
                CheapVec::from_vec(vec![
                    sea_orm::Value::from(token.to_string()),
                    sea_orm::Value::from(user_id.to_string()),
                    sea_orm::Value::from(Utc::now().naive_utc()),
                ]),
            ))
            .await
            .map_err(|err| anyhow!("Query execution error: {}", err))?;

        Ok(token)
    }

    async fn invalidate(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        user_id: UserId,
    ) -> Result<()> {
        todo!()
    }

    async fn remove_expired(&self, db_conn: Arc<dyn AnyDatabaseConnection>) -> Result<()> {
        todo!()
    }
}

#[typetag::serde(name = "MySQLRole")]
#[async_trait]
impl AnyRoleMethod for MySQLRole {
    fn name(&self) -> &'static str {
        "mysqlrole"
    }

    fn db_id(&self) -> Option<CompactString> {
        self.database_id.to_owned()
    }

    async fn get(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        user_id: UserId,
    ) -> Result<Option<CompactString>> {
        todo!()
    }

    async fn set(
        &self,
        db_conn: Arc<dyn AnyDatabaseConnection>,
        user_id: UserId,
        role: CompactString,
    ) -> Result<()> {
        todo!()
    }

    async fn remove(&self, db_conn: Arc<dyn AnyDatabaseConnection>, user_id: UserId) -> Result<()> {
        todo!()
    }
}
