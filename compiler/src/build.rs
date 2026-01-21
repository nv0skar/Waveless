// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

///
/// The Waveless' project's builder.
/// The builder tasks are:
/// 1. Serialize the `config.toml` file.
/// 2. Load user's endpoints.
/// 3. Discover the endpoints (optional).
/// 4. Hash the current state of the databases (optional).
/// 5. Build and serialize the project's binary file.
///
use crate::*;

/// Build the project in the current path (if no `config.toml` file is present in the current directory it will be searched in parent directories)
#[instrument(skip_all)]
pub fn build<T: 'static>() -> Result<Box<dyn Any>> {
    let config = config_loader::project_config()?;

    debug!(
        "Started building at {} with the following settings {:#?}.",
        chrono::Local::now(),
        config
    );

    // Deserialize user's endpoints
    let mut endpoints = endpoint::Endpoints::new(CheapVec::new());
    {
        let endpoints_dir = get_project_root()?.join(config.compiler().endpoints_dir());

        let endpoints_path = read_dir(endpoints_dir)
            .context("Unexpected error, the endpoints directory cannot be listed.")?;

        for endpoint_path in endpoints_path {
            let endpoint_path = endpoint_path?;
            match read(endpoint_path.path()) {
                Ok(file_buffer) => {
                    match toml::from_slice::<endpoint::Endpoints>(&file_buffer) {
                        Ok(new_endpoints) => {
                            println!("{:?}", endpoint_path.path());
                            endpoints.merge(new_endpoints)?
                        }
                        Err(err) => {
                            Err(anyhow!(
                                "Cannot deserialize the endpoints definition file '{}'.%{}",
                                endpoint_path.file_name().display(),
                                err.to_string()
                            ))?;
                        }
                    };
                }
                Err(err) => {
                    Err(anyhow!(
                        "Cannot open the endpoints definition file '{}'.%{}",
                        endpoint_path.file_name().display(),
                        err.to_string()
                    ))?;
                }
            }
        }

        debug!("Deserialized user's endpoints: {:#?}", endpoints);
    }

    // Serialize the project's build
    let build = binary::Build::new(
        config.general().to_owned(),
        config.server().to_owned(),
        endpoints,
        CheapVec::new(),
    );

    if TypeId::of::<T>() == TypeId::of::<Bytes>() {
        let buff = build.encode_binary()?;

        debug!(
            "Finished building project successfully at {}.",
            chrono::Local::now(),
        );

        Ok(Box::new(buff))
    } else if TypeId::of::<T>() == TypeId::of::<binary::Build>() {
        Ok(Box::new(build))
    } else {
        panic!("Unexpected type.")
    }
}

/// Generates the binary's file from the provided buffer.
pub fn binary_file_from_buff(buff: Bytes) -> Result<ResultContext> {
    let target_file: PathBuf;

    // Set the build file's name a combination of its CRC32 hash and the current timestamp
    let build_name = format!(
        "{}_{}.wv",
        chrono::Local::now().format("%d_%m_%Y_%H_%M"),
        crc32fast::hash(buff.as_slice())
    );

    if let Ok(_) = create_dir(get_project_root()?.join("target")) {
        debug!("`target` directory does't exist, a new one will be created.")
    };

    target_file = get_project_root()?.join("target").join(build_name);

    write(target_file.to_owned(), buff)?;

    debug!("Emitted build file on {}", target_file.display());

    Ok(format!(
        "'{}' has been built at {}",
        config_loader::project_config()?.general().name(),
        target_file
            .file_name()
            .ok_or(anyhow!("No build file name."))?
            .display()
    )
    .to_compact_string())
}
