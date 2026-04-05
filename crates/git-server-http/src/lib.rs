pub mod error;
pub mod handlers;

use std::sync::Arc;

use axum::{Router, routing::get, routing::post};
use git_server_core::discovery::RepoStore;

pub type SharedState = Arc<RepoStore>;

pub fn router(store: RepoStore) -> Router {
    let state: SharedState = Arc::new(store);
    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/", get(handlers::list_repos))
        // {*repo_path} is a catch-all; suffixes /info/refs and /git-upload-pack
        // are stripped from the path inside the handler by dispatching via
        // separate routes that embed the fixed suffix literally after the
        // catch-all -- which axum disallows.  Instead we use two separate
        // catch-all routes distinguished by their terminal component.
        .route("/{*path}", get(handlers::info_refs_dispatch))
        .route("/{*path}", post(handlers::upload_pack_dispatch))
        .with_state(state)
}
