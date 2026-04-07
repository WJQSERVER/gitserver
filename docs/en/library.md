# Library Usage Guide

`gitserver-core` and `gitserver-http` can be embedded as libraries in larger Rust applications, integrating Git Smart HTTP serving into existing systems.

## Add Dependencies

```toml
[dependencies]
gitserver-core = { git = "https://github.com/WJQSERVER/gitserver" }
gitserver-http = { git = "https://github.com/WJQSERVER/gitserver" }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## Quick Start: Discovery Mode

The simplest usage scans a directory and auto-discovers bare repositories:

```rust
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Scan ./repos directory, max depth 3
    let store = RepoStore::discover("./repos".into(), 3)?;

    // Create shared state and build routes
    let state = SharedState::new(store);
    let app = router(state);

    // Start serving
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## Configure Authentication

```rust
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router, AuthConfig, BasicAuthConfig};

let store = RepoStore::discover("./repos".into(), 3)?;

let state = SharedState::with_auth(store, AuthConfig {
    basic: Some(BasicAuthConfig {
        username: "admin".into(),
        password: "secret".into(),
    }),
    bearer_token: Some("my-token".into()),
});

let app = router(state);
```

Basic and Bearer authentication can be configured simultaneously -- either one passing is sufficient.

## Configure Service Policy

```rust
use gitserver_http::{SharedState, ServicePolicy, AuthConfig};

let state = SharedState::with_store_and_auth_policy(
    store,
    AuthConfig::default(),
    ServicePolicy {
        upload_pack: true,       // clone/fetch
        upload_pack_v2: true,    // protocol v2
        receive_pack: true,      // push (disabled by default)
    },
);
```

## Dynamic Mode: Manual Repository Registration

Suited for multi-tenant scenarios where filesystem scanning is not desired:

```rust
use std::sync::Arc;
use gitserver_core::discovery::{DynamicRepoRegistry, MutableRepoRegistry, RepoInfo};
use gitserver_http::{SharedState, router, ServicePolicy, AuthConfig};

// Create empty registry
let registry = Arc::new(DynamicRepoRegistry::new());

let state = SharedState::with_registry(
    registry.clone(),
    AuthConfig::default(),
    ServicePolicy::default(),
);

// Register a repository (validates that the path is a bare repo)
registry.register(RepoInfo {
    name: "my-project.git".into(),
    relative_path: "tenant-a/my-project.git".into(),
    absolute_path: "/data/repos/tenant-a/my-project.git".into(),
    description: Some("My project".into()),
})?;

// Unregister a repository
registry.unregister("tenant-a/my-project.git")?;

let app = router(state);
```

A convenience method is also available:

```rust
let state = SharedState::with_dynamic_registry(
    AuthConfig::default(),
    ServicePolicy::default(),
);

// Register via state
state.register_repo(RepoInfo {
    name: "project.git".into(),
    relative_path: "project.git".into(),
    absolute_path: "/data/repos/project.git".into(),
    description: None,
})?;
```

## Embed into an Existing Axum Router

The `router()` function returns a standard Axum `Router` that can be nested into a larger application:

```rust
use axum::Router;
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router};

let store = RepoStore::discover("./repos".into(), 3)?;
let git_state = SharedState::new(store);
let git_app = router(git_state);

// Mount under /git
let app = Router::new()
    .nest("/git", git_app)
    // ... other routes
    ;
```

## Use Low-Level Handlers

For finer control over request handling, call functions from the `handlers` module directly:

```rust
use axum::http::{HeaderMap, StatusCode};
use gitserver_core::discovery::RepoStore;
use gitserver_http::{
    SharedState, ServicePolicy, AuthConfig,
    handlers::{info_refs_endpoint, ServiceKind},
};

let store = RepoStore::discover("./repos".into(), 3)?;
let state = SharedState::with_store_and_auth_policy(
    store,
    AuthConfig::default(),
    ServicePolicy::default(),
);

// Call info/refs handler directly
let response = info_refs_endpoint(
    &state,
    "my-project.git",
    ServiceKind::UploadPack,
    HeaderMap::new(),
).await?;
```

## Background Periodic Refresh

Discovery mode supports runtime refresh of the repository list:

```rust
use tokio::time::{interval, Duration, MissedTickBehavior};

// Clone state for the background task
let refresh_state = state.clone();
tokio::spawn(async move {
    let mut ticker = interval(Duration::from_secs(30));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;
        if let Err(e) = refresh_state.refresh().await {
            tracing::warn!("refresh failed: {e}");
        }
    }
});
```

## Using gitserver-core Directly

If you only need Git protocol operations without the HTTP layer, use `gitserver-core` directly:

```rust
use gitserver_core::{
    discovery::RepoStore,
    backend::GitBackend,
    pack::{UploadPackRequest, UploadPackCapabilities, ShallowRequest},
};

// Repository discovery
let store = RepoStore::discover("./repos".into(), 3)?;
let repo = store.resolve("my-project.git")?;

// Ref advertisement
let backend = GitBackend::new(repo.absolute_path.clone());
let refs = backend.advertise_refs()?;

// Generate a pack
let request = UploadPackRequest {
    wants: vec![/* object ids */],
    haves: vec![],
    done: true,
    capabilities: UploadPackCapabilities::default(),
    shallow: ShallowRequest::default(),
    object_ids: None,
};
let pack_stream = backend.upload_pack(&request).await?;
```
