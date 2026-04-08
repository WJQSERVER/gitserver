# 库调用指南

`gitserver-core` 和 `gitserver-http` 可以作为库嵌入到更大的**Rust**应用中, 将 Git Smart HTTP 服务集成到现有系统.

## 添加依赖

```toml
[dependencies]
gitserver-core = { git = "https://github.com/WJQSERVER/gitserver" }
gitserver-http = { git = "https://github.com/WJQSERVER/gitserver" }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## 快速开始: 发现模式

最简单的用法是扫描一个目录, 自动发现其中的裸仓库:

```rust
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 扫描 ./repos 目录, 最大深度 3
    let store = RepoStore::discover("./repos".into(), 3)?;

    // 创建共享状态并构建路由
    let state = SharedState::new(store);
    let app = router(state);

    // 启动服务
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("install Ctrl+C handler");
        })
        .await?;

    Ok(())
}
```

如果你暴露了 `GET /healthz`, 它在正常运行时返回 `200`, 在通过 `SharedState::start_shutdown()` 标记为排空后返回 `503`. 这样宿主应用可以先切换 readiness, 再等待已在进行中的请求完成.

## 配置认证

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

Basic 和 Bearer 认证可以同时配置, 任一通过即可.

## 配置服务策略

```rust
use gitserver_http::{SharedState, ServicePolicy, AuthConfig};

let state = SharedState::with_store_and_auth_policy(
    store,
    AuthConfig::default(),
    ServicePolicy {
        upload_pack: true,       // clone/fetch
        upload_pack_v2: true,    // 协议 v2
        receive_pack: true,      // push (默认关闭)
    },
);
```

## 动态模式: 手动注册仓库

适合多租户场景, 不需要文件系统扫描:

```rust
use std::sync::Arc;
use gitserver_core::discovery::{DynamicRepoRegistry, MutableRepoRegistry, RepoInfo};
use gitserver_http::{SharedState, router, ServicePolicy, AuthConfig};

// 创建空注册表
let registry = Arc::new(DynamicRepoRegistry::new());

let state = SharedState::with_registry(
    registry.clone(),
    AuthConfig::default(),
    ServicePolicy::default(),
);

// 注册仓库 (会验证路径是否为裸仓库)
registry.register(RepoInfo {
    name: "my-project.git".into(),
    relative_path: "tenant-a/my-project.git".into(),
    absolute_path: "/data/repos/tenant-a/my-project.git".into(),
    description: Some("My project".into()),
})?;

// 注销仓库
registry.unregister("tenant-a/my-project.git")?;

let app = router(state);
```

也可以使用快捷方法:

```rust
let state = SharedState::with_dynamic_registry(
    AuthConfig::default(),
    ServicePolicy::default(),
);

// 通过 state 注册
state.register_repo(RepoInfo {
    name: "project.git".into(),
    relative_path: "project.git".into(),
    absolute_path: "/data/repos/project.git".into(),
    description: None,
})?;
```

## 嵌入到现有 Axum 路由

`router()` 返回的标准 Axum `Router` 可以嵌套到更大的应用中:

```rust
use axum::Router;
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router};

let store = RepoStore::discover("./repos".into(), 3)?;
let git_state = SharedState::new(store);
let git_app = router(git_state);

// 挂载到 /git 路径下
let app = Router::new()
    .nest("/git", git_app)
    // ... 其他路由
    ;
```

## 直接使用公开处理器

如果你已经在用 Axum, 想自行组装部分 Git 路由, 可以直接调用 `handlers` 模块里公开的处理函数:

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

// 直接调用 info/refs 处理器
let response = info_refs_endpoint(
    &state,
    "my-project.git",
    ServiceKind::UploadPack,
    HeaderMap::new(),
).await?;
```

## 后台定期刷新

发现模式支持运行时刷新仓库列表; 动态模式不需要, 也不支持 `refresh()`:

```rust
use tokio::time::{interval, Duration, MissedTickBehavior};

// 克隆 state 用于后台任务
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

## 直接使用 gitserver-core

如果只需要 Git 协议操作而不需要 HTTP 层, 可以直接使用 `gitserver-core`:

```rust
use gitserver_core::{
    discovery::RepoStore,
    backend::GitBackend,
    pack::{UploadPackRequest, UploadPackCapabilities, ShallowRequest},
};

// 仓库发现
let store = RepoStore::discover("./repos".into(), 3)?;
let repo = store.resolve("my-project.git")?;

// 引用通告
let backend = GitBackend::new(repo.absolute_path.clone());
let refs = backend.advertise_refs()?;

// 生成 pack
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
