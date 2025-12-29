// Waveless
// Copyright (C) 2025 Oscar Alvarez Gonzalez

///
/// The Waveless's project's 'config.toml' file will be divided into: compiler settings, runtime settings, authentication and database authentication credentials
/// Both Authentication and CheapVec<DatabaseAuth> will be shared with the compiler and the runtime.
///
/// TODO: maybe implement default variants
///
use crate::*;

/// Includes all the project's config
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[serde(default)]
pub struct Project {
    general: General,
    compiler: Compiler,
    server: Server,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            general: Default::default(),
            compiler: Default::default(),
            server: Default::default(),
        }
    }
}

#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[serde(default)]
/// General settings that will be shared across Waveless's components
pub struct General {
    /// contains all project's databases
    databases: CheapVec<DatabaseConfig>,

    /// contains authentication settings
    authentication: Authentication,

    /// contains admin settings
    admin: Admin,
}

impl Default for General {
    fn default() -> Self {
        Self {
            databases: CheapVec::from_vec(vec![
                Default::default(),
                DatabaseConfig {
                    id: "secondary".to_compact_string(),
                    is_primary: false,
                    connection: DatabaseConnection::ExternalModule {
                        id: "custom_database_driver".to_compact_string(),
                        connection: "...".to_compact_string(),
                    },
                    checksum_schema: false,
                    pool_size: None,
                },
            ]),
            authentication: Default::default(),
            admin: Default::default(),
        }
    }
}

/// Compiler settings: these parameters will be used by the API compiler exclusively
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[serde(default)]
pub struct Compiler {
    /// this option defines the compiler's strategy to analyze the data schema.
    /// if set to `None`, the compiler will only include the user defined endpoints
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    endpoint_discovery: Option<DataSchemaDiscoveryConfig>,

    /// display all the endpoints while building
    display_endpoints_on_compile: bool,

    /// this is the directory where all the user defined endpoints will be located
    endpoints_dir: CompactString,

    /// this is the directory where scripts that may be used to create the db, make migrations... are located
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    bootstrap_scripts_dir: Option<CompactString>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            endpoint_discovery: Some(Default::default()),
            display_endpoints_on_compile: true,
            endpoints_dir: "./endpoints/".to_compact_string(),
            bootstrap_scripts_dir: Some("./bootstrap/".to_compact_string()),
        }
    }
}

/// Runtime settings: these parameters will be used by the server exclusively
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[serde(default)]
pub struct Server {
    /// can be set through cli parameters or env variables
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    listening_addr: Option<SocketAddr>,

    /// the files on the specified path will be served
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    static_files: Option<CompactString>,

    /// prefix for all api endpoints
    api_prefix: CompactString,

    /// the compiler will generate a checksum of the schema of each database, if this option is marked, the server executor will check whether the checksum on each start
    check_databases_cheksums: bool, // TODO

    /// set the http cache time header
    http_cache_time: usize,
}

impl Default for Server {
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

/// Defines parameters to be used by the data schema discovery
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[serde(default)]
pub struct DataSchemaDiscoveryConfig {
    /// strategy to discover endpoints
    method: DataSchemaDiscoveryMethod,

    /// identifier of the database to analyze.
    /// if it is `None` the primary database will be used
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    database_id: Option<DatabaseId>,
}

impl Default for DataSchemaDiscoveryConfig {
    fn default() -> Self {
        Self {
            method: Default::default(),
            database_id: Some("main".to_compact_string()),
        }
    }
}

/// Defines every available strategy to discover endpoints
#[derive(Clone, PartialEq, Serialize, Deserialize, Display, Debug)]
#[cfg_attr(feature = "toml_codec", serde(tag = "type"))]
pub enum DataSchemaDiscoveryMethod {
    /// The MySQL discovey strategy will analyze a MySQL database in order to generate a representation of the data model that will be analyzed by the endpoint generator backend
    #[display("MySQL schema discovery (skipping: {:?})", skip_tables)]
    MySQL {
        #[cfg_attr(
            feature = "toml_codec",
            serde(skip_serializing_if = "CheapVec::is_empty")
        )]
        skip_tables: CheapVec<CompactString>, // Do not forget that auth, session and role tables are also skipped
    },
    #[display("{:?}: {:?}", id, config)]
    ExternalModule {
        id: DataSchemaDiscoveryMethodId,
        config: HashMap<CompactString, Bytes>,
    },
}

impl Default for DataSchemaDiscoveryMethod {
    fn default() -> Self {
        Self::MySQL {
            skip_tables: CheapVec::from_vec(vec!["_private_table".to_compact_string()]),
        }
    }
}

/// Defines a database to be used by Waveless
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[serde(default)]
pub struct DatabaseConfig {
    /// unique identifier of the database
    id: DatabaseId,

    /// indicates whether this database is primary (no need to set database id on auth, session and role storage)
    is_primary: bool,

    /// holds the database type, the address and the credentials
    connection: DatabaseConnection,

    /// whether or not to checksum the database schema on build
    checksum_schema: bool,

    /// defines the maximum number of simultaneous connections, by default this will be twice the number of available cores
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
    pool_size: Option<usize>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            id: "main".to_compact_string(),
            is_primary: true,
            connection: Default::default(),
            checksum_schema: true,
            pool_size: None,
        }
    }
}

/// Defines credentials for all database backends
#[derive(Clone, PartialEq, Serialize, Deserialize, Display, Debug)]
#[cfg_attr(feature = "toml_codec", serde(tag = "type"))]
pub enum DatabaseConnection {
    // TODO - Support more authentication methods
    /// MySQL database
    #[display("MySQL: {}@{} on {}", username, host, db)]
    MySQL {
        host: SocketAddr,
        username: CompactString,
        password: CompactString,
        db: CompactString,
    },
    /// TODO: load custom database drivers
    #[display("{:?}: {}", id, connection)]
    ExternalModule {
        id: ExternalDriverId,
        connection: CompactString,
    },
}

impl Default for DatabaseConnection {
    fn default() -> Self {
        Self::MySQL {
            host: SocketAddr::new("127.0.0.1".parse().unwrap(), 3306),
            username: "example_user".to_compact_string(),
            password: "example_password".to_compact_string(),
            db: "example_db".to_compact_string(),
        }
    }
}

/// Defines how the server executor can handle authentication
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[serde(default)]
pub struct Authentication {
    /// whether authentication is enabled
    enabled: bool,

    /// all the available methods to authenticate
    methods: CheapVec<AuthenticationMethod>,

    /// session token config
    session: Session,

    /// users' role config
    roles: Roles,

    /// whether to allow user registration
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
#[serde(default)]
pub struct Admin {
    /// whether to enable the admin panel
    enable_panel: bool, // TODO

    /// all roles that are considered admins
    allowed_roles: CheapVec<CompactString>,

    /// whether to gather statistics or not
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
#[cfg_attr(feature = "toml_codec", serde(tag = "type"))]
pub enum AuthenticationMethod {
    #[display("Name & password authentication on SQL using table {}", table_name)]
    SqlNamePassword {
        /// will use the primary database by default
        #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
        database_id: Option<DatabaseId>,
        table_name: CompactString,
        /// this field references to the user table in order to model a relationship and implement login with name, emails, IDs... Must not be primary key.
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
#[serde(default)]
pub struct Session {
    /// defines how the sessions' token will be stored
    storage: SessionStorage,

    /// max age of sessions
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
#[serde(default)]
pub struct Roles {
    /// defines how the roles will be stored
    storage: RoleStorage,

    /// default role when users sign up
    #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
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
#[cfg_attr(feature = "toml_codec", serde(tag = "type"))]
pub enum SessionStorage {
    /// Note that a single user may have many tokens
    #[display("SQL backed token on table {}", table_name)]
    SqlToken {
        /// will use the primary database by default
        #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
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
#[cfg_attr(feature = "toml_codec", serde(tag = "type"))]
pub enum RoleStorage {
    /// Note that a single user may have multiple roles
    #[display("SQL backed users' roles check on {}", table_name)]
    SqlUser {
        /// will use the primary database by default
        #[cfg_attr(feature = "toml_codec", serde(skip_serializing_if = "Option::is_none"))]
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
#[cfg(feature = "toml_codec")]
mod tests {
    use super::*;

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
