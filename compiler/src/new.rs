// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

//!
//!  Handles the new project creation.
//!
use crate::*;

/// Create a new project in the current dir with the specified name
#[instrument(skip_all)]
pub fn new_project(name: CompactString) -> Result<ResultContext> {
    // Create the default `project.toml` file.
    let default_project = project::Project::default();

    // Create the project's folder.
    let project_path = current_dir()?.join(&name);

    {
        if let Err(err) = create_dir(&project_path) {
            Err(anyhow!(
                "Cannot create project's folder {}. Are you sure that there is no project with the same name and that you have write permissions?%{}",
                name,
                err.to_string().blue()
            ))?;
        }

        debug!("Created project's folder at {}.", project_path.display());
    }

    // Serialize default `project.toml` file.
    {
        let mut config_file = File::create_new(project_path.join("project.toml"))
            .context("Unexpected error, cannot create `project.toml` file.")?;

        let _ = config_file.write(toml::to_string_pretty(&default_project)?.as_bytes())?;

        debug!("Loaded default `project.toml` file.");
    }

    // Generate all subfolders.
    {
        create_dir(
            project_path.join(
                default_project
                    .compiler()
                    .bootstrap_scripts_dir()
                    .to_owned()
                    .unwrap_or("bootstrap".into()),
            ),
        )?;

        create_dir(project_path.join(default_project.compiler().endpoints_dir()))?;

        create_dir(
            project_path.join(
                default_project
                    .compiler()
                    .hooks_dir()
                    .to_owned()
                    .unwrap_or("hooks".into()),
            ),
        )?;

        create_dir(project_path.join(".discovered_endpoints"))?;

        create_dir(project_path.join("target"))?;

        debug!("Created project directories.");
    }

    // Serialize the a sample endpoint.
    {
        let endpoints = Endpoints::new_unchecked(CheapVec::from_vec(vec![
            Endpoint::new(
                "ListProducts".into(),
                "/products/{size}".into(),
                Some("v1".into()),
                HttpMethod::Get,
                Default::default(),
                Some(Arc::new(MySQLExecute::from(MySQLQueryVariants::new_raw(
                    "SELECT * FROM products WHERE size = {size}".into(),
                )))),
                Some("Get all the products by the given size.".into()),
                CheapVec::from_vec(vec!["products".into()]),
                Default::default(),
                Default::default(),
                false,
                false,
                Default::default(),
                false,
                false,
                false,
            ),
            Endpoint::new(
                "ListPosts".into(),
                "posts".into(),
                Some("v1".into()),
                HttpMethod::Get,
                None,
                Some(Arc::new(MySQLExecute::from(MySQLQueryVariants::new_raw(
                    "SELECT * FROM posts".into(),
                )))),
                Some("Get all posts.".into()),
                CheapVec::from_vec(vec!["posts".into()]),
                Default::default(),
                Default::default(),
                false,
                false,
                Default::default(),
                false,
                false,
                false,
            ),
        ]));

        let mut sample_endpoint_file = File::create_new(
            project_path
                .join(default_project.compiler().endpoints_dir())
                .join("sample_endpoint.toml"),
        )
        .context("Unexpected error, cannot create `sample_endpoint.toml` file.")?;

        let _ = sample_endpoint_file.write(toml::to_string_pretty(&endpoints)?.as_bytes())?;
    }

    Ok(format!(
        "New project '{}' was created at '{}' with a default '{}' and a sample endpoint at '{}'.",
        name,
        project_path.display(),
        "project.toml",
        Path::new(default_project.compiler().endpoints_dir())
            .join("sample_endpoint.toml")
            .display()
    )
    .into())
}
