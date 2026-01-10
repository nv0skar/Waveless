// Waveless
// Copyright (C) 2025 Oscar Alvarez Gonzalez

///
///  Handles the new project creation.
///
use crate::*;

/// Create a new project in the current dir with the specified name
#[instrument(skip_all)]
pub fn new_project(name: CompactString) -> Result<()> {
    // Create the default `config.toml` file
    let default_project = project::Project::default();

    // Create the project's folder
    let project_path = current_dir()?.join(&name);

    {
        if let Err(err) = create_dir(project_path.to_owned()) {
            println!(
                "{} {}",
                "ERROR:".bright_red().bold(),
                format!("Cannot create project's folder {}. ({:?})", name, err).bright_white()
            );
            println!("{}", "❓ Are you sure that there is no project with the same name and that you have write permissions?".bright_white());
            exit(1);
        }

        debug!("Created project's folder at {}.", project_path.display());
    }

    // Serialize default `config.toml` file
    {
        let mut config_file = File::create_new(project_path.join("config.toml"))
            .context("Unexpected error, cannot create `config.toml` file.")?;

        let _ = config_file.write(toml::to_string_pretty(&default_project)?.as_bytes())?;

        debug!("Loaded default `config.toml` file.");
    }

    // Generate all subfolders
    {
        create_dir(
            project_path.join(
                default_project
                    .compiler()
                    .bootstrap_scripts_dir()
                    .to_owned()
                    .unwrap_or("bootstrap".to_compact_string()),
            ),
        )?;

        create_dir(project_path.join(default_project.compiler().endpoints_dir().to_owned()))?;

        create_dir(
            project_path.join(
                default_project
                    .compiler()
                    .hooks_dir()
                    .to_owned()
                    .unwrap_or("hooks".to_compact_string()),
            ),
        )?;

        create_dir(project_path.join(".discovered_endpoints"))?;

        create_dir(project_path.join("target"))?;

        debug!("Created project directories.");
    }

    // Serialize the a sample endpoint
    {
        let endpoints = endpoint::Endpoints::new(CheapVec::from_vec(vec![
            endpoint::Endpoint::default(),
            endpoint::Endpoint::new(
                "posts".to_compact_string(),
                Some("v1".to_compact_string()),
                endpoint::HttpMethod::Get,
                None,
                Some(endpoint::Executor::SQL {
                    query: "SELECT * FROM posts".to_compact_string(),
                }),
                Some("Get all posts.".to_compact_string()),
                CheapVec::from_vec(vec!["posts".to_compact_string()]),
                Default::default(),
                Default::default(),
                Default::default(),
                false,
                Default::default(),
                false,
                false,
            ),
        ]));

        let mut sample_endpoint_file = File::create_new(
            project_path
                .join(default_project.compiler().endpoints_dir().to_owned())
                .join("sample_endpoint.toml"),
        )
        .context("Unexpected error, cannot create `sample_endpoint.toml` file.")?;

        let _ = sample_endpoint_file.write(toml::to_string_pretty(&endpoints)?.as_bytes())?;
    }

    println!(
        "{}",
        format!(
            "✅ New project `{}` was created at `{}` with a default `{}` and a sample endpoint at `{}`.",
            name,
            project_path.display(),
            "config.toml",
            Path::new(default_project.compiler().endpoints_dir()).join("sample_endpoint.toml").display()
        )
        .bold()
        .bright_white()
    );

    Ok(())
}
