// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

//!
//! The Waveless' project's builder.
//! The builder tasks are:
//! 1. Serialize the `project.toml` file.
//! 2. Load user's endpoints.
//! 3. Discover the endpoints (optional).
//! 4. Hash the current state of the databases (optional).
//! 5. Build and serialize the project's binary file.
//!
use crate::*;

use generator::*;

/// Builds the project in the current path (if no `project.toml` file is present in the current directory it will be searched in parent directories)
#[instrument(skip_all)]
pub async fn load<T: 'static>() -> Result<Either<ObjectArtifact, Bytes>> {
    let cx = CompilerCx::acquire();

    let project = cx.project();
    let workspace_root = cx.workspace_root();

    debug!(
        "Started building at {} with the following settings {:#?}.",
        chrono::Local::now(),
        project
    );

    // Deserializes user's endpoints.
    let mut endpoints = Endpoints::new_unchecked(CheapVec::new_const());
    {
        let endpoints_dir = workspace_root.join(project.compiler().endpoints_dir());

        let endpoints_path = read_dir(endpoints_dir)
            .context("Unexpected error, the endpoints directory cannot be listed.")?;

        for endpoint_path in endpoints_path {
            let endpoint_path = endpoint_path?;
            match read(endpoint_path.path()) {
                Ok(file_buffer) => {
                    match toml::from_slice::<Endpoints>(&file_buffer) {
                        Ok(new_endpoints) => endpoints.merge(new_endpoints)?,
                        Err(err) => Err(err).wrap_err(format!(
                            "Cannot deserialize the endpoints definition file '{}'.",
                            endpoint_path.file_name().display(),
                        ))?,
                    };
                }
                Err(err) => Err(err).wrap_err(format!(
                    "Cannot open the endpoints definition file '{}'.",
                    endpoint_path.file_name().display(),
                ))?,
            }
        }

        debug!("Deserialized user's endpoints: {:#?}", endpoints);
    }

    // Discovers the endpoints and checksums the database's schema.
    let generated_endpoints = GeneratedEndpoints::generate().await?;

    if create_dir(workspace_root.join(".generated_endpoints")).is_ok() {
        debug!("'.generated_endpoints' directory does't exist, a new one will be created.")
    };

    let mut endpoint_generator_checksums = CheapVec::new_const();

    for (generator_id, (new_endpoints, checksum)) in generated_endpoints.get() {
        let target_file = workspace_root
            .join(".generated_endpoints")
            .join(format!("{}.toml", hex::encode(generator_id)));

        write(
            &target_file,
            toml::to_string_pretty(&new_endpoints)?.as_bytes(),
        )?;

        info!(
            "Generated endpoints from `{}` were dumped into `{}` ({}).",
            hex::encode(generator_id),
            target_file.display(),
            String::from_utf8(generator_id.to_vec()).unwrap_or("?".into())
        );

        if let Some(checksum) = checksum {
            endpoint_generator_checksums.push(checksum.to_owned());
        }

        endpoints.merge(new_endpoints.to_owned())?;
    }

    // Serializes the project's build.
    let build = ObjectArtifact::new(
        project.config().to_owned(),
        project.server().to_owned(),
        endpoints,
        endpoint_generator_checksums,
    );

    if TypeId::of::<T>() == TypeId::of::<Bytes>() {
        let buff = build.encode_binary()?;

        debug!(
            "Finished building project successfully at {}.",
            chrono::Local::now(),
        );

        Ok(Right(buff))
    } else if TypeId::of::<T>() == TypeId::of::<ObjectArtifact>() {
        Ok(Left(build))
    } else {
        panic!("Unexpected type.")
    }
}

/// Generates the binary's file from the provided buffer.
pub fn binary_file_from_buff(buff: Bytes) -> Result<ResultContext> {
    let cx = CompilerCx::acquire();

    let project = cx.project();
    let workspace_root = cx.workspace_root();

    // Set the build file's name a combination of its CRC32 hash and the current timestamp
    let build_name = format!(
        "{}_{}_{}.wv",
        project.config().name(),
        chrono::Local::now().format("%d_%m_%Y_%H_%M"),
        crc32fast::hash(buff.as_slice())
    );

    let target_file = workspace_root.join("target").join(build_name);

    if create_dir(workspace_root.join("target")).is_ok() {
        debug!("`target` directory doesn't exist, a new one will be created.")
    };

    write(&target_file, buff)?;

    debug!("Emitted build file on {}", target_file.display());

    Ok(format!(
        "'{}' has been built at {}",
        project.config().name(),
        target_file
            .file_name()
            .ok_or(eyre!("No build file name."))?
            .display()
    )
    .into())
}
