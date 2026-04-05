pub mod error;
pub mod handlers;

use std::sync::Arc;

use axum::{Router, routing::get, routing::post};
use git_server_core::discovery::RepoStore;
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
    store: Arc<RwLock<RepoStore>>,
    auth: AuthConfig,
}

impl SharedState {
    pub fn new(store: RepoStore) -> Self {
        Self::with_auth(store, AuthConfig::default())
    }

    pub fn with_auth(store: RepoStore, auth: AuthConfig) -> Self {
        Self {
            store: Arc::new(RwLock::new(store)),
            auth,
        }
    }

    pub async fn list(&self) -> Vec<git_server_core::discovery::RepoInfo> {
        self.store.read().await.list().to_vec()
    }

    pub async fn resolve(
        &self,
        relative: &str,
    ) -> git_server_core::error::Result<git_server_core::discovery::RepoInfo> {
        self.store.read().await.resolve(relative).cloned()
    }

    pub async fn refresh(&self) -> git_server_core::error::Result<()> {
        self.store.write().await.refresh()
    }

    pub fn auth(&self) -> &AuthConfig {
        &self.auth
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
