// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

///
/// -- The Waveless's binary format --
/// Waveless follows a compiler-like design:
/// - There is the frontend/compiler, which generates the binary that represents the whole api: all endpoints all server configuration, database checksums...
///   This is built upon user-defined endpoints and automatically generated endpoints from various discovery strategies.
/// - The server executor/runtime, which loads the Waveless's project's binary and serves all endpoints, static files, interfaces with every database, generates statistics, manages authentication and session logic, admin panel...
/// Note that when the binary is built, the magic bytes are appended to the beginning of the file
///
use crate::*;

/// The project's build file
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct ProjectBuild {
    /// contains general settings shared with the frontend/compiler
    general: project::General,

    /// specific compiler settings
    compiler_settings: project::Compiler,

    /// defines all the API endpoints
    endpoints: endpoint::Endpoints,

    /// contains all the databases' checksum
    databases_checksums: CheapVec<DatabaseChecksum>,
}

impl ProjectBuild {
    /// Serializes the binary and appends the magic bytes to the beginning of the buffer
    pub fn encode_binary(&self) -> Result<Bytes> {
        let mut buffer = self.encode()?;
        buffer.insert_from_slice(0, BINARY_MAGIC);
        Ok(buffer)
    }

    /// Removes the magic bytes from the beginning of the file and deserializes the binary
    pub fn decode_binary(buffer: &Bytes) -> Result<Self> {
        Ok(ProjectBuild::decode(&buffer[(BINARY_MAGIC.len() - 1)..])?)
    }
}

/// Default implementation for testing and validation
impl Default for ProjectBuild {
    fn default() -> Self {
        Self {
            general: Default::default(),
            compiler_settings: Default::default(),
            endpoints: endpoint::Endpoints::default(),
            databases_checksums: CheapVec::from_vec(vec![Default::default()]),
        }
    }
}

/// Matches a database with it's checksum
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct DatabaseChecksum {
    /// identifier of the database, if it is `None` the primary database will be used
    database_id: Option<DatabaseId>,
    checksum: Bytes,
}

/// Default implementation for testing and validation
impl Default for DatabaseChecksum {
    fn default() -> Self {
        Self {
            database_id: None,
            checksum: CheapVec::from_elem(0, 8),
        }
    }
}

/// Note that for this test both `waveless_config`'s and `waveless_schema`'s `postcard_codec` flag is set, this will be disable some Serde attributes as skipping field serialization and enum type format.
#[cfg(test)]
mod tests {
    use super::*;

    use rustyrosetta::codec::*;

    #[test]
    fn default_into_bin_and_back() -> Result<()> {
        let build = ProjectBuild::default();

        let serialized = build.encode().context("Cannot serialize project build.")?;

        let deserialized =
            ProjectBuild::decode(&serialized).context("Cannot deserialize project build. Did you disable the `toml_codec` flag on `waveless_config` and `waveless_schema`?")?;

        assert_eq!(build, deserialized);

        println!("{:#?}\n", build);
        println!("{:?}", serialized);

        Ok(())
    }
}
