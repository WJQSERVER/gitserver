use std::path::PathBuf;

use clap::Parser;
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
        let app = git_server_http::router(store);
        let addr = format!("{}:{}", cli.bind, cli.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!(%addr, "server listening");
        axum::serve(listener, app).await?;
        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}
