pub mod error;
pub mod handlers;

use std::sync::Arc;

use axum::{Router, routing::get, routing::post};
use gitserver_core::discovery::{
    DynamicRepoRegistry, MutableRepoRegistry, RepoResolver, RepoStore,
};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct AuthConfig {
    pub basic: Option<BasicAuthConfig>,
    pub bearer_token: Option<String>,
}

#[derive(Clone)]
pub struct BasicAuthConfig {
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct SharedState {
    mode: RepoMode,
    auth: AuthConfig,
    policy: ServicePolicy,
}

#[derive(Clone)]
enum RepoMode {
    Discovered(Arc<RwLock<RepoStore>>),
    Dynamic {
        resolver: Arc<dyn RepoResolver>,
        registry: Arc<dyn MutableRepoRegistry>,
    },
}

#[derive(Clone)]
pub struct ServicePolicy {
    pub upload_pack: bool,
    pub upload_pack_v2: bool,
    pub receive_pack: bool,
}

impl Default for ServicePolicy {
    fn default() -> Self {
        Self {
            upload_pack: true,
            upload_pack_v2: true,
            receive_pack: false,
        }
    }
}

impl SharedState {
    pub fn new(store: RepoStore) -> Self {
        Self::with_store_and_auth_policy(store, AuthConfig::default(), ServicePolicy::default())
    }

    pub fn with_auth(store: RepoStore, auth: AuthConfig) -> Self {
        Self::with_store_and_auth_policy(store, auth, ServicePolicy::default())
    }

    pub fn with_store_and_auth_policy(
        store: RepoStore,
        auth: AuthConfig,
        policy: ServicePolicy,
    ) -> Self {
        let store: Arc<RwLock<RepoStore>> = Arc::new(RwLock::new(store));
        Self {
            mode: RepoMode::Discovered(store),
            auth,
            policy,
        }
    }

    pub fn with_registry(
        registry: Arc<dyn MutableRepoRegistry>,
        auth: AuthConfig,
        policy: ServicePolicy,
    ) -> Self {
        Self {
            mode: RepoMode::Dynamic {
                resolver: registry.clone(),
                registry,
            },
            auth,
            policy,
        }
    }

    pub fn with_dynamic_registry(auth: AuthConfig, policy: ServicePolicy) -> Self {
        let registry: Arc<DynamicRepoRegistry> = Arc::new(DynamicRepoRegistry::new());
        Self::with_registry(registry, auth, policy)
    }

    pub async fn list(
        &self,
    ) -> gitserver_core::error::Result<Vec<gitserver_core::discovery::RepoInfo>> {
        match &self.mode {
            RepoMode::Discovered(store) => Ok(store.read().await.list().to_vec()),
            RepoMode::Dynamic { resolver, .. } => resolver.list(),
        }
    }

    pub async fn resolve(
        &self,
        relative: &str,
    ) -> gitserver_core::error::Result<gitserver_core::discovery::RepoInfo> {
        match &self.mode {
            RepoMode::Discovered(store) => store.read().await.resolve(relative).cloned(),
            RepoMode::Dynamic { resolver, .. } => resolver.resolve(relative),
        }
    }

    pub async fn refresh(&self) -> gitserver_core::error::Result<()> {
        match &self.mode {
            RepoMode::Discovered(store) => {
                let (root, max_depth) = {
                    let guard = store.read().await;
                    (guard.root().to_path_buf(), guard.max_depth())
                };
                let refreshed =
                    tokio::task::spawn_blocking(move || RepoStore::discover(root, max_depth))
                        .await
                        .map_err(|e| {
                            gitserver_core::error::Error::Protocol(format!(
                                "refresh task panicked: {e}"
                            ))
                        })??;
                *store.write().await = refreshed;
                Ok(())
            }
            RepoMode::Dynamic { .. } => Err(gitserver_core::error::Error::Protocol(
                "refresh is only available in discovery mode".into(),
            )),
        }
    }

    pub fn register_repo(
        &self,
        repo: gitserver_core::discovery::RepoInfo,
    ) -> gitserver_core::error::Result<()> {
        match &self.mode {
            RepoMode::Dynamic { registry, .. } => registry.register(repo),
            RepoMode::Discovered(_) => Err(gitserver_core::error::Error::Protocol(
                "dynamic registry is not enabled".into(),
            )),
        }
    }

    pub fn unregister_repo(&self, relative: &str) -> gitserver_core::error::Result<()> {
        match &self.mode {
            RepoMode::Dynamic { registry, .. } => registry.unregister(relative),
            RepoMode::Discovered(_) => Err(gitserver_core::error::Error::Protocol(
                "dynamic registry is not enabled".into(),
            )),
        }
    }

    pub fn auth(&self) -> &AuthConfig {
        &self.auth
    }

    pub fn policy(&self) -> &ServicePolicy {
        &self.policy
    }
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/", get(handlers::list_repos))
        // {*repo_path} is a catch-all; suffixes /info/refs and /git-upload-pack
        // are stripped from the path inside the handler by dispatching via
        // separate routes that embed the fixed suffix literally after the
        // catch-all -- which axum disallows.  Instead we use two separate
        // catch-all routes distinguished by their terminal component.
        .route("/{*path}", get(handlers::info_refs_dispatch))
        .route("/{*path}", post(handlers::rpc_dispatch))
        .with_state(state)
}
