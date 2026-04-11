use std::path::PathBuf;

use clap::Parser;
use tokio::sync::{oneshot, watch};
use tokio::time::{Duration, MissedTickBehavior};
use tracing::info;

use gitserver_core::discovery::RepoStore;

const PRE_STOP_DRAIN_DELAY: Duration = Duration::from_secs(2);
const RUNTIME_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Parser)]
#[command(
    name = "gitserver",
    version,
    about = "Standalone smart HTTP Git server"
)]
struct Cli {
    /// Root directory containing bare Git repositories
    root: PathBuf,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,

    /// Port number
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: tracing::Level,

    /// Log format: text or json
    #[arg(long, default_value = "text")]
    log_format: LogFormat,

    /// Number of Tokio worker threads
    #[arg(short, long)]
    workers: Option<usize>,

    /// Max directory depth for repo discovery
    #[arg(long, default_value_t = 3)]
    max_depth: u32,

    /// Periodically rescan repositories to pick up additions/removals.
    #[arg(long, default_value_t = 30)]
    rescan_interval_secs: u64,

    /// Require HTTP Basic auth with this username.
    #[arg(long, requires = "auth_basic_password")]
    auth_basic_username: Option<String>,

    /// Require HTTP Basic auth with this password.
    #[arg(long, requires = "auth_basic_username")]
    auth_basic_password: Option<String>,

    /// Require Bearer authentication with this token.
    #[arg(long)]
    auth_bearer_token: Option<String>,

    /// Enable git-receive-pack and allow push operations.
    #[arg(long, default_value_t = false)]
    enable_receive_pack: bool,

    /// Timeout for upload-pack and receive-pack requests in seconds.
    #[arg(long, default_value_t = 300)]
    request_timeout_secs: u64,

    /// Maximum uncompressed pack bytes allowed for upload-pack responses.
    #[arg(long)]
    max_pack_bytes: Option<u64>,
}

#[derive(Clone, clap::ValueEnum)]
enum LogFormat {
    Text,
    Json,
}

fn init_tracing(level: tracing::Level, format: &LogFormat) {
    let env_filter = tracing_subscriber::EnvFilter::new(level.to_string());
    match format {
        LogFormat::Text => {
            tracing_subscriber::fmt().with_env_filter(env_filter).init();
        }
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .json()
                .init();
        }
    }
}

async fn shutdown_signal() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = signal(SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result?,
            _ = terminate.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
    }

    Ok(())
}

fn spawn_rescan_task(
    state: gitserver_http::SharedState,
    interval_secs: u64,
    mut shutdown_rx: watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        interval.tick().await;

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    break;
                }
                _ = interval.tick() => {
                    match state.refresh().await {
                        Ok(()) => tracing::debug!("repository list refreshed"),
                        Err(err) => tracing::warn!("failed to refresh repository list: {err}"),
                    }
                }
            }
        }

        tracing::debug!("repository rescan task stopped");
    })
}

async fn shutdown_sequence(
    state: gitserver_http::SharedState,
    shutdown_tx: watch::Sender<bool>,
    graceful_tx: oneshot::Sender<()>,
) {
    shutdown_sequence_with_signal(state, shutdown_tx, graceful_tx, shutdown_signal()).await;
}

async fn shutdown_sequence_with_signal<F>(
    state: gitserver_http::SharedState,
    shutdown_tx: watch::Sender<bool>,
    graceful_tx: oneshot::Sender<()>,
    signal: F,
) where
    F: std::future::Future<Output = anyhow::Result<()>>,
{
    match signal.await {
        Ok(()) => {}
        Err(err) => {
            // If signal registration fails, keep serving instead of treating setup failure
            // as an actual shutdown request.
            tracing::error!("failed to install shutdown signal handler: {err}");
            return;
        }
    }

    info!(
        delay_secs = PRE_STOP_DRAIN_DELAY.as_secs_f32(),
        "shutdown signal received, entering draining mode"
    );
    state.start_shutdown();
    let _ = shutdown_tx.send(true);

    tokio::time::sleep(PRE_STOP_DRAIN_DELAY).await;

    info!("pre-stop drain window elapsed, stopping listener");
    // This is best-effort: if axum has already exited, dropping the send result is fine.
    let _ = graceful_tx.send(());
}

async fn await_graceful_shutdown(graceful_rx: oneshot::Receiver<()>) {
    if graceful_rx.await.is_err() {
        // Keep the shutdown future pending if the sender disappears unexpectedly so the
        // server does not stop listening without an explicit graceful shutdown signal.
        std::future::pending::<()>().await;
    }
}

async fn run_server(cli: Cli, store: RepoStore) -> anyhow::Result<()> {
    let auth = gitserver_http::AuthConfig {
        basic: cli
            .auth_basic_username
            .zip(cli.auth_basic_password)
            .map(|(username, password)| gitserver_http::BasicAuthConfig { username, password }),
        bearer_token: cli.auth_bearer_token,
    };
    let state = gitserver_http::SharedState::with_store_and_auth_policy(
        store,
        auth,
        gitserver_http::ServicePolicy {
            receive_pack: cli.enable_receive_pack,
            request_timeout: Duration::from_secs(cli.request_timeout_secs),
            max_pack_bytes: cli.max_pack_bytes,
            ..Default::default()
        },
    );
    let app = gitserver_http::router(state.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let rescan_handle = spawn_rescan_task(state.clone(), cli.rescan_interval_secs, shutdown_rx);
    let (graceful_tx, graceful_rx) = oneshot::channel();

    let addr = format!("{}:{}", cli.bind, cli.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "server listening");

    // Run signal handling independently so the server can continue serving until the
    // shutdown sequence explicitly flips readiness and releases the graceful receiver.
    tokio::spawn(shutdown_sequence(state, shutdown_tx, graceful_tx));

    axum::serve(listener, app)
        .with_graceful_shutdown(await_graceful_shutdown(graceful_rx))
        .await?;

    match tokio::time::timeout(RUNTIME_SHUTDOWN_TIMEOUT, rescan_handle).await {
        Ok(Ok(())) => tracing::debug!("repository rescan task joined cleanly"),
        Ok(Err(err)) => return Err(err.into()),
        Err(_) => {
            tracing::warn!(
                timeout_secs = RUNTIME_SHUTDOWN_TIMEOUT.as_secs_f32(),
                "timed out waiting for repository rescan task to stop"
            );
        }
    }

    info!("server shutdown complete");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.log_level, &cli.log_format);

    if !cli.root.is_dir() {
        anyhow::bail!("root path '{}' is not a directory", cli.root.display());
    }

    let store = RepoStore::discover(cli.root.clone(), cli.max_depth)?;
    let repos = store.list();
    info!(count = repos.len(), "discovered repositories");
    for repo in repos {
        info!(name = %repo.name, path = %repo.relative_path, "found repository");
    }

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    if let Some(workers) = cli.workers {
        builder.worker_threads(workers);
    }
    builder.enable_all();
    let runtime = builder.build()?;

    let result = runtime.block_on(run_server(cli, store));
    runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT);
    result?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use gitserver_http::router;
    use http_body_util::BodyExt;
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn create_bare_repo(root: &std::path::Path, name: &str) {
        let repo_path = root.join(name);
        let out = std::process::Command::new("git")
            .args(["init", "--bare", repo_path.to_str().unwrap()])
            .output()
            .expect("git init --bare failed");
        assert!(out.status.success(), "git init --bare failed: {out:?}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn shutdown_sequence_exposes_draining_before_listener_stop() {
        let tmp = TempDir::new().unwrap();
        create_bare_repo(tmp.path(), "alpha.git");
        let store = RepoStore::discover(tmp.path().to_path_buf(), 1).unwrap();
        let state = gitserver_http::SharedState::new(store);
        let app = router(state.clone());

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let (graceful_tx, mut graceful_rx) = oneshot::channel();
        let (signal_tx, signal_rx) = oneshot::channel::<()>();

        tokio::spawn(shutdown_sequence_with_signal(
            state,
            shutdown_tx,
            graceful_tx,
            async move {
                let _ = signal_rx.await;
                Ok(())
            },
        ));

        signal_tx.send(()).unwrap();

        shutdown_rx.changed().await.unwrap();
        assert!(*shutdown_rx.borrow());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "shutting_down");

        tokio::time::sleep(PRE_STOP_DRAIN_DELAY / 2).await;
        assert!(graceful_rx.try_recv().is_err());

        tokio::time::sleep(PRE_STOP_DRAIN_DELAY).await;
        assert!(graceful_rx.await.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn shutdown_sequence_does_not_drain_when_signal_setup_fails() {
        let tmp = TempDir::new().unwrap();
        create_bare_repo(tmp.path(), "alpha.git");
        let store = RepoStore::discover(tmp.path().to_path_buf(), 1).unwrap();
        let state = gitserver_http::SharedState::new(store);

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let (graceful_tx, mut graceful_rx) = oneshot::channel();

        shutdown_sequence_with_signal(state.clone(), shutdown_tx, graceful_tx, async {
            Err(anyhow::anyhow!("boom"))
        })
        .await;

        assert!(!state.is_draining());
        assert!(!*shutdown_rx.borrow());
        assert!(graceful_rx.try_recv().is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn await_graceful_shutdown_ignores_dropped_sender() {
        let (_graceful_tx, graceful_rx) = oneshot::channel::<()>();

        let waiter = tokio::spawn(await_graceful_shutdown(graceful_rx));
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(!waiter.is_finished());
        waiter.abort();
        let _ = waiter.await;
    }
}
