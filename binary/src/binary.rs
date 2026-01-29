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
pub struct Build {
    /// Contains general settings shared with the frontend/compiler.
    general: project::General,

    /// Specific server settings.
    server_settings: project::Server,

    /// Defines all the API endpoints.
    endpoints: Endpoints,

    /// Contains all the databases' checksum.
    /// TODO: in the future there will be a method to checksum all the database's
    /// schema regardless of whether they have been 'discovered'.
    databases_checksums: CheapVec<DatabaseChecksum, 0>,
}

impl Build {
    /// Serializes the binary and appends the magic bytes to the beginning of the buffer.
    /// NOTE: the `BINARY_MODE` flag is set as a workaround of the issue https://github.com/serde-rs/serde/issues/1732,
    /// so now we can safely serialize all repository's structures and enums regardless whether the serializer being
    /// self-descriptive or not.
    pub fn encode_binary(&self) -> Result<Bytes> {
        BINARY_MODE.set(true);
        debug!(
            "Binary mode is set, as the serializer requires `#[serde(skip_serializing_if = '...')]` to be disabled."
        );
        let mut buffer = self.encode()?;
        buffer.insert_from_slice(0, BINARY_MAGIC);
        BINARY_MODE.set(false);
        Ok(buffer)
    }

    /// Removes the magic bytes from the beginning of the file and deserializes the binary.
    pub fn decode_binary(buffer: &Bytes) -> Result<Self> {
        Build::decode(&buffer[BINARY_MAGIC.len()..])
    }
}

/// Default implementation for testing and validation.
impl Default for Build {
    fn default() -> Self {
        Self {
            general: Default::default(),
            server_settings: Default::default(),
            endpoints: Endpoints::new(CheapVec::from_vec(vec![Endpoint::default()])),
            databases_checksums: CheapVec::new(),
        }
    }
}

/// Matches a database with it's checksum
#[derive(Clone, PartialEq, Constructor, Serialize, Deserialize, Getters, Debug)]
#[getset(get = "pub")]
pub struct DatabaseChecksum {
    /// identifier of the database
    database_id: DatabaseId,
    checksum: Bytes,
}

/// Default implementation for testing and validation
impl Default for DatabaseChecksum {
    fn default() -> Self {
        Self {
            database_id: "None".to_compact_string(),
            checksum: CheapVec::from_elem(0, 8),
        }
    }
}

/// Note that for this test both `waveless_config`'s and `waveless_schema`'s `postcard_codec` flag is set, this will be disable some Serde attributes as skipping field serialization and enum type format.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_into_bin_and_back() -> Result<()> {
        let build = Build::default();

        let serialized = build
            .encode_binary()
            .context("Cannot serialize project build.")?;

        let deserialized =
            Build::decode_binary(&serialized).context("Cannot deserialize project build. Did you disable the `toml_codec` flag on `waveless_config` and `waveless_schema`?")?;

        assert_eq!(build, deserialized);

        println!("Build file structure: {:#?}\n", build);
        println!("Binary representation: {:?}", serialized);

        Ok(())
    }
}
