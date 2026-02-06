// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

//!
//! The Waveless's project's 'project.toml' file will be divided into: compiler settings, runtime settings, authentication and database authentication credentials
//! Both Authentication and CheapVec<DatabaseAuth> will be shared with the compiler and the runtime.
//!
//! TODO: maybe implement default variants
//!
use std::fmt::Debug;

use crate::*;

use build::*;
use databases::*;

/// Includes all the project's config
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Project {
    #[serde(flatten)]
    config: Config,
    compiler: Compiler,
    server: Executor,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            config: Default::default(),
            compiler: Default::default(),
            server: Default::default(),
        }
    }
}

#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
/// General settings that will be shared across Waveless's components
pub struct Config {
    /// Project's name.
    name: CompactString,

    /// Contains all project's databases.
    #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    databases: CheapVec<DatabaseConfig, 0>,

    /// Contains authentication settings.
    authentication: Authentication,

    /// Contains admin settings.
    admin: Admin,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "Example".to_compact_string(),
            databases: CheapVec::from_vec(vec![
                Default::default(),
                DatabaseConfig {
                    id: "secondary".to_compact_string(),
                    is_primary: false,
                    connection: Arc::new(ExternalDBConnectionConfig {
                        id: "custom_database_driver".to_compact_string(),
                        connection: "...".to_compact_string(),
                    }),
                    schema_discovery: None,
                    pool_min_size: None,
                    pool_max_size: None,
                },
            ]),
            authentication: Default::default(),
            admin: Default::default(),
        }
    }
}

/// Compiler settings: these parameters will be used by the API's compiler exclusively
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Compiler {
    /// this is the directory where all the user defined endpoints will be located
    endpoints_dir: CompactString,

    /// this is the directory where all the endpoint's hooks will be located
    hooks_dir: Option<CompactString>,

    /// this is the directory where scripts that may be used to create the db, make migrations... are located
    #[serde(default, skip_serializing_if = "should_skip_option")]
    bootstrap_scripts_dir: Option<CompactString>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            endpoints_dir: "./endpoints/".to_compact_string(),
            hooks_dir: Some("./hooks/".to_compact_string()),
            bootstrap_scripts_dir: Some("./bootstrap/".to_compact_string()),
        }
    }
}

/// Runtime settings: these parameters will be used by the server exclusively
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Executor {
    /// can be set through cli parameters or env variables
    #[serde(default, skip_serializing_if = "should_skip_option")]
    listening_addr: Option<SocketAddr>,

    /// the files on the specified path will be served
    #[serde(default, skip_serializing_if = "should_skip_option")]
    static_files: Option<CompactString>,

    /// prefix for all api endpoints
    api_prefix: CompactString,

    /// the compiler will generate a checksum of the schema of each database, if this option is marked, the server executor will check whether the checksum on each start
    check_databases_cheksums: bool,

    /// set the http cache time header
    http_cache_time: usize,
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            listening_addr: Some(SocketAddr::new("127.0.0.1".parse().unwrap(), 8080)),
            static_files: Some("./static/".to_compact_string()),
            api_prefix: "/api".to_compact_string(),
            check_databases_cheksums: true,
            http_cache_time: 0,
        }
    }
}

/// Defines a database to be used by Waveless
#[derive(Clone, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct DatabaseConfig {
    /// Unique identifier of the database.
    id: DatabaseId,

    /// Indicates whether this database is primary (no need to set database id on auth, session and role storage).
    is_primary: bool,

    /// Defines credentials for all database backends.
    connection: Arc<dyn AnyDatabaseConnectionConfig>,

    /// Defines the compiler's strategy to analyze the databases' data schema.
    /// NOTE: there might be many different types that implement the
    /// `AnyDataSchemaDiscoveryMethod` trait for a single database type.
    /// For example, given a single database type (like MySQL), there might be an
    /// ad-hoc schema discovery implementation and a simple endpoint geneator,
    /// also, there might be a more complex `AnyDataSchemaDiscoveryMethod` that
    /// chains the internal MySQL schema analyzer and enhances the endpoint generation.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    schema_discovery: Option<DataSchemaDiscoveryConfig>,

    /// Defines the minimum number of simultaneous connections, by default this will be half the `pool_max_size`.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    pool_min_size: Option<usize>,

    /// Defines the maximum number of simultaneous connections, by default this will be twice the number of available cores.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    pool_max_size: Option<usize>,
}

impl PartialEq for DatabaseConfig {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            id: "main".to_compact_string(),
            is_primary: true,
            connection: Arc::new(databases::mysql::MySQLDBConnectionConfig::new(
                SocketAddr::new("127.0.0.1".parse().unwrap(), 3306),
                "example_user".to_compact_string(),
                "example_password".to_compact_string(),
                "example_db".to_compact_string(),
            )),
            schema_discovery: Some(Default::default()),
            pool_min_size: Some(std::thread::available_parallelism().unwrap().get() * 2),
            pool_max_size: Some(std::thread::available_parallelism().unwrap().get() * 2),
        }
    }
}

/// TODO: add documentation.
#[typetag::serde]
#[async_trait]
pub trait AnyDatabaseConnectionConfig: Any + BoxedAny + DynClone + Send + Sync + Debug {
    async fn new_conn(
        &self,
        id: CompactString,
        pool_min_size: Option<usize>,
        pool_max_size: Option<usize>,
    ) -> Result<(Arc<dyn AnyDatabaseConnection>, Box<dyn Any>)>;
}

/// TODO: load custom database drivers.
#[derive(Clone, Serialize, Deserialize, Display, Debug)]
#[display("{:?}: {}", id, connection)]
pub struct ExternalDBConnectionConfig {
    id: ExternalDriverId,
    connection: CompactString,
}

boxed_any!(ExternalDBConnectionConfig);

impl PartialEq for ExternalDBConnectionConfig {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[typetag::serde]
#[async_trait]
impl AnyDatabaseConnectionConfig for ExternalDBConnectionConfig {
    async fn new_conn(
        &self,
        _id: CompactString,
        _pool_min_size: Option<usize>,
        _pool_max_size: Option<usize>,
    ) -> Result<(Arc<dyn AnyDatabaseConnection>, Box<dyn Any>)> {
        todo!("Not yet implemented.");
    }
}

/// Defines parameters to be used by the data schema discovery
#[derive(Clone, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct DataSchemaDiscoveryConfig {
    /// Strategy to discover endpoints.
    method: Arc<dyn AnyDataSchemaDiscoveryMethod>,

    /// Generate endpoints from the database's schema if marked.
    generate_endpoints: bool,

    // Whether to checksum the database's schema.
    checksum: bool,
}

impl Default for DataSchemaDiscoveryConfig {
    fn default() -> Self {
        Self {
            method: Arc::new(schema::mysql::MySQLSchemaDiscoveryMethod::new(
                CheapVec::from_vec(vec!["_private_table".to_compact_string()]),
            )),
            generate_endpoints: true,
            checksum: true,
        }
    }
}

/// TODO: add documentation.
#[typetag::serde]
#[async_trait]
pub trait AnyDataSchemaDiscoveryMethod: Any + BoxedAny + DynClone + Send + Sync + Debug {
    async fn schema(
        &self,
        db_id: CompactString,
        db_config: Arc<dyn AnyDatabaseConnectionConfig>,
    ) -> Result<(Box<dyn Any>, DatabaseChecksum)>;
}

/// The external module will use the project's hooks tp establish a database connection.
/// TODO: load custom schema discovery drivers.
#[derive(Clone, Serialize, Deserialize, Display, Debug)]
#[display("{:?}: {:?}", id, config)]
pub struct ExternalSchemaDiscoveryMethod {
    id: DataSchemaDiscoveryMethodId,
    config: HashMap<CompactString, Bytes>,
}

boxed_any!(ExternalSchemaDiscoveryMethod);

impl PartialEq for ExternalSchemaDiscoveryMethod {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[typetag::serde]
#[async_trait]
impl AnyDataSchemaDiscoveryMethod for ExternalSchemaDiscoveryMethod {
    async fn schema(
        &self,
        _db_id: CompactString,
        _db_conn_config: Arc<dyn AnyDatabaseConnectionConfig>,
    ) -> Result<(Box<dyn Any>, DatabaseChecksum)> {
        todo!("Not yet implemented.")
    }
}

/// Defines how the server executor can handle authentication
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Authentication {
    /// Whether authentication is enabled.
    enabled: bool,

    /// All the available methods to authenticate.
    #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    methods: CheapVec<AuthenticationMethod, 0>,

    /// Session token config.
    session: Session,

    /// Users' role config.
    roles: Roles,

    /// Whether to allow user registration.
    allow_registration: bool,
}

impl Default for Authentication {
    fn default() -> Self {
        Self {
            enabled: true,
            methods: CheapVec::from_vec(vec![
                Default::default(),
                AuthenticationMethod::ExternalModule {
                    id: "ldap_example_server".to_compact_string(),
                    config: "...".to_compact_string(),
                },
            ]),
            session: Default::default(),
            roles: Default::default(),
            allow_registration: true,
        }
    }
}

/// Defines admin settings and privileges on the server.
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Admin {
    /// Whether to enable the admin panel.
    enable_panel: bool, // TODO

    /// All roles that are considered admins.
    #[serde(default, skip_serializing_if = "should_skip_cheapvec")]
    allowed_roles: CheapVec<CompactString, 0>,

    /// Whether to gather statistics or not.
    statistics: bool, // TODO
}

impl Default for Admin {
    fn default() -> Self {
        Self {
            enable_panel: true,
            allowed_roles: CheapVec::from_vec(vec!["admin".to_compact_string()]),
            statistics: false,
        }
    }
}

/// Defines all the available user authentication mechanisms.
/// Note that the auth data does not have to live in a SQL database...
#[derive(Clone, PartialEq, Serialize, Deserialize, Display, Debug)]
pub enum AuthenticationMethod {
    #[display("Name & password authentication on SQL using table {}", table_name)]
    SqlNamePassword {
        /// Will use the primary database by default.
        #[serde(default, skip_serializing_if = "should_skip_option")]
        database_id: Option<DatabaseId>,
        table_name: CompactString,
        /// This field references to the user table in order to model a relationship and implement login with name, emails, IDs... Must not be primary key.
        user_field: CompactString,
        password_field: CompactString,
        totp_field: Option<CompactString>,
    },
    // TODO - Passkey authentication
    #[display("{:?}: {}", id, config)]
    ExternalModule {
        id: ExternalDriverId,
        config: CompactString,
    },
}

impl Default for AuthenticationMethod {
    fn default() -> Self {
        Self::SqlNamePassword {
            database_id: None,
            table_name: "users_auth".to_compact_string(),
            user_field: "user_id".to_compact_string(),
            password_field: "password_id".to_compact_string(),
            totp_field: None,
        }
    }
}

/// Session token configuration
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Session {
    /// Defines how the sessions' token will be stored.
    storage: SessionStorage,

    /// Max age of sessions.
    max_age: usize,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            storage: Default::default(),
            max_age: 86400,
        }
    }
}

/// Role configuration
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct Roles {
    /// Defines how the roles will be stored.
    storage: RoleStorage,

    /// Default role when users sign up.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    default_role: Option<CompactString>,
}

impl Default for Roles {
    fn default() -> Self {
        Self {
            storage: Default::default(),
            default_role: None,
        }
    }
}

/// Defines the backing storage of the session token
#[derive(Clone, PartialEq, Serialize, Deserialize, Display, Debug)]
pub enum SessionStorage {
    /// Note that a single user may have many tokens
    #[display("SQL backed token on table {}", table_name)]
    SqlToken {
        /// Will use the primary database by default.
        #[serde(default, skip_serializing_if = "should_skip_option")]
        database_id: Option<DatabaseId>,
        table_name: CompactString,
        /// Must not be primary key.
        user_field: CompactString,
        token_field: CompactString,
        created_field: CompactString,
    },
    #[display("{:?}: {}", id, config)]
    ExternalModule {
        id: ExternalDriverId,
        config: CompactString,
    },
}

impl Default for SessionStorage {
    fn default() -> Self {
        Self::SqlToken {
            database_id: None,
            table_name: "sesions".to_compact_string(),
            user_field: "user_id".to_compact_string(),
            token_field: "token".to_compact_string(),
            created_field: "created_at".to_compact_string(),
        }
    }
}

/// Defines all the availables ways of checking users' roles
#[derive(Clone, PartialEq, Serialize, Deserialize, Display, Debug)]
pub enum RoleStorage {
    /// Note that a single user may have multiple roles
    #[display("SQL backed users' roles check on {}", table_name)]
    SqlUser {
        /// Will use the primary database by default.
        #[serde(default, skip_serializing_if = "should_skip_option")]
        database_id: Option<DatabaseId>,
        table_name: CompactString,
        /// Must not be primary key.
        user_field: CompactString,
        role_field: CompactString,
    },

    #[display("{:?}: {}", id, config)]
    ExternalModule {
        id: ExternalDriverId,
        config: CompactString,
    },
}

impl Default for RoleStorage {
    fn default() -> Self {
        Self::SqlUser {
            database_id: None,
            table_name: "sesions".to_compact_string(),
            user_field: "user_id".to_compact_string(),
            role_field: "role".to_compact_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::*;

    #[test]
    fn default_into_toml_and_back() -> Result<()> {
        let project_config = Project::default();

        let serialized = toml::to_string_pretty(&project_config)
            .context("Cannot serialize default project config into TOML.")?;
        let deserialized = toml::from_str::<Project>(&serialized)
            .context("Cannot deserialize default TOML config.")?;

        assert_eq!(project_config, deserialized);

        println!("{:#?}\n", project_config);
        println!("{}", serialized);
        Ok(())
    }
}
