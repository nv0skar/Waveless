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
        let cx = Self::new(RwLock::new(build), EndpointRouter::new(), None);
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
                    let cx = Self::new(RwLock::new(build), EndpointRouter::new(), Some(path));
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
    pub async fn build_router(&self) -> Result<()> {
        let Self { build, router, .. } = self;

        let prefix = build.read().await.executor().api_prefix().to_owned();

        let mut endpoints = build.read().await.endpoints().inner().to_owned();

        // Add authentication endpoints if enabled.
        if let Some(auth_config) = build.read().await.config().authentication().to_owned() {
            for (kind, endpoint) in INTERNAL_ENDPOINTS.iter() {
                if let InternalEndpointKind::Authentication = kind {
                    // Check whether we are trying to add the signup endpoint while being disabled.
                    if !auth_config.allow_signup() && endpoint.id() == SIGNUP_ENDPOINT_ID {
                        continue;
                    }

                    endpoints.push(endpoint.to_owned());
                }
            }
        }

        // Add all other internal endpoints.
        INTERNAL_ENDPOINTS.iter().for_each(|(kind, endpoint)| {
            if let InternalEndpointKind::Other = kind {
                endpoints.push(endpoint.to_owned());
            }
        });

        // Reset the router to prevent deleted endpoints to persist.
        for method_router in router.to_owned().iter() {
            router.remove(method_router.key());
        }

        // Add all endpoints to the router.
        let prefix = prefix.trim_matches('/');

        for endpoint in endpoints {
            let route_parts: CheapVec<Option<CompactString>, 3> = CheapVec::from_buf([
                Some(prefix.into()),
                endpoint
                    .version()
                    .to_owned()
                    .map(|version| version.trim_matches('/').into()),
                Some(endpoint.route().trim_matches('/').into()),
            ]);

            let route = route_parts.into_iter().flatten().join_compact("/");

            if let Some(mut router) = router.get_mut(endpoint.method()) {
                let _ = router.insert(route, endpoint.to_owned()); // the error here is ignored.
            } else {
                let mut new_router = Router::new();
                new_router.insert(route, endpoint.to_owned())?;

                let _ = router.insert(endpoint.method().to_owned(), new_router);
            }
        }

        Ok(())
    }
}
