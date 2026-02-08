// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

#[derive(Constructor, Getters, Debug)]
#[getset(get = "pub")]
pub struct RuntimeCx {
    build: RwLock<ExecutorBuild>,
    router: EndpointRouter,
    _loaded_from: Option<PathBuf>,
}

impl RuntimeCx {
    pub fn acquire() -> &'static Self {
        RUNTIME_CX
            .get()
            .ok_or(anyhow!("Runtime's context should have been initialized."))
            .unwrap()
    }

    /// Sets the `RUNTIME_CX`'s `OnceLock`.
    /// NOTE: If runtime's context is set this method will panic.
    pub fn set_cx(self) {
        if !RUNTIME_CX.initialized() {
            RUNTIME_CX.set(self).unwrap();
        } else {
            panic!("Runtime's context has already been initialized.");
        }
    }

    /// Builds the runtime's context by loading the project's build
    /// from the given **build** and building the router.
    pub async fn from_build(build: ExecutorBuild) -> Result<Self> {
        let mut cx = Self::new(RwLock::new(build), EndpointRouter::new(), None);
        cx.build_router().await?;
        Ok(cx)
    }

    /// Builds the runtime's context by loading the project's build
    /// from the given **path** and building the router.
    pub async fn from_path(path: PathBuf) -> Result<Self> {
        match read(path.to_owned()) {
            Ok(file_buffer) => match ExecutorBuild::decode_binary(&CheapVec::from_vec(file_buffer))
            {
                Ok(build) => {
                    let mut cx = Self::new(RwLock::new(build), EndpointRouter::new(), Some(path));
                    cx.build_router().await?;
                    Ok(cx)
                }
                Err(err) => Err(anyhow!(
                    "Cannot deserialize the project's binary '{}'.%{}",
                    path.display(),
                    err.to_string()
                )),
            },
            Err(err) => Err(anyhow!(
                "Cannot open '{}'. Are you sure that you have the file's permissions?%{}",
                path.display(),
                err.to_string()
            )),
        }
    }

    /// Builds the endpoint router from the given runtime's context.
    async fn build_router(&mut self) -> Result<()> {
        let Self { build, router, .. } = self;

        let prefix = build.read().await.executor().api_prefix().to_owned();

        let mut endpoints = build.read().await.endpoints().inner().to_owned();

        // Adds authentication endpoints if enabled.
        let auth_config = build.read().await.config().authentication().to_owned();

        if let Some(_) = auth_config {
            // Adds the login endpoint at /{api_prefix}/internal/login
            let login_endpoint = Endpoint::new(
                LOGIN_ENDPOINT_ID.to_compact_string(),
                "login".to_compact_string(),
                Some("internal".to_compact_string()),
                HttpMethod::Post,
                None,
                None,
                Some("Login a user capturing all parameters and forwading them to the underlying authentication method.".to_compact_string()),
                CheapVec::new(),
                CheapVec::new(),
                CheapVec::new(),
                false,
                CheapVec::new(),
                true,
                false,
                true,
            );

            endpoints.push(login_endpoint);
        }

        for endpoint in endpoints {
            let mut full_route = PathBuf::new();
            full_route.push(prefix.trim_matches('/'));
            if let Some(prefix) = endpoint.version() {
                full_route.push(prefix.trim_matches('/'));
            }
            full_route.push(endpoint.route().trim_matches('/'));

            if let Some(mut router) = router.get_mut(endpoint.method()) {
                router.insert(full_route.display().to_string(), endpoint.to_owned())?;
            } else {
                let mut new_router = Router::new();
                new_router.insert(full_route.display().to_string(), endpoint.to_owned())?;
                let _ = router.insert(endpoint.method().to_owned(), new_router);
            }
        }

        Ok(())
    }
}
