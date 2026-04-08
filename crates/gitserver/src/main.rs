use std::path::PathBuf;

use clap::Parser;
use tokio::sync::watch;
use tokio::time::{Duration, MissedTickBehavior};
use tracing::info;

use gitserver_core::discovery::RepoStore;

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
            ..Default::default()
        },
    );
    let app = gitserver_http::router(state.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let rescan_handle = spawn_rescan_task(state.clone(), cli.rescan_interval_secs, shutdown_rx);

    let addr = format!("{}:{}", cli.bind, cli.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(%addr, "server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            if let Err(err) = shutdown_signal().await {
                tracing::error!("failed to install shutdown signal handler: {err}");
            }
            info!("shutdown signal received, draining in-flight requests");
            state.start_shutdown();
            let _ = shutdown_tx.send(true);
        })
        .await?;

    rescan_handle.await?;
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

    runtime.block_on(run_server(cli, store))?;

    Ok(())
}
