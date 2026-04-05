use std::path::PathBuf;

use clap::Parser;
use tokio::time::{Duration, MissedTickBehavior};
use tracing::info;

use git_server_core::discovery::RepoStore;

#[derive(Parser)]
#[command(
    name = "git-server",
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

    runtime.block_on(async {
        let auth = git_server_http::AuthConfig {
            basic: cli
                .auth_basic_username
                .zip(cli.auth_basic_password)
                .map(|(username, password)| git_server_http::BasicAuthConfig { username, password }),
            bearer_token: cli.auth_bearer_token,
        };
        let state = git_server_http::SharedState::with_store_and_auth_policy(
            store,
            auth,
            git_server_http::ServicePolicy {
                receive_pack: cli.enable_receive_pack,
                ..Default::default()
            },
        );
        let app = git_server_http::router(state.clone());

        let interval_secs = cli.rescan_interval_secs;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            interval.tick().await;

            loop {
                interval.tick().await;
                match state.refresh().await {
                    Ok(()) => tracing::debug!("repository list refreshed"),
                    Err(err) => tracing::warn!("failed to refresh repository list: {err}"),
                }
            }
        });

        let addr = format!("{}:{}", cli.bind, cli.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!(%addr, "server listening");
        axum::serve(listener, app).await?;
        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}
