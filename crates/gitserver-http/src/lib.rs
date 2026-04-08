pub mod error;
pub mod handlers;
pub mod state;

use axum::{Router, routing::get, routing::post};
pub use state::{AuthConfig, BasicAuthConfig, ServicePolicy, SharedState};

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
