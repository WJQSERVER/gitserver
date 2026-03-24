use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use git_server_bench::fixtures;
use git_server_core::discovery::RepoStore;
use std::net::SocketAddr;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

fn bench_git_clone(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("git_clone");

    for (name, repo_path) in [
        ("small", fixtures::small_repo()),
        ("medium", fixtures::medium_repo()),
        ("large", fixtures::large_repo()),
    ] {
        let (addr, shutdown_tx, handle) = rt.block_on(start_server(repo_path));
        let url = format!("http://{}/repo.git", addr);

        group.bench_with_input(BenchmarkId::new("clone", name), &url, |b, url| {
            b.iter(|| {
                let dest = TempDir::new().unwrap();
                let output = Command::new("git")
                    .args(["clone", url, dest.path().to_str().unwrap()])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .output()
                    .unwrap();
                assert!(output.status.success(), "git clone failed");
            });
        });

        let _ = shutdown_tx.send(());
        rt.block_on(async {
            let _ = handle.await;
        });
    }

    group.finish();
}

async fn start_server(
    repo_path: &Path,
) -> (SocketAddr, oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    let store = RepoStore::discover(repo_path.parent().unwrap().to_path_buf(), 0).unwrap();
    let router = git_server_http::router(store);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await
            .unwrap();
    });
    (addr, tx, handle)
}

criterion_group!(benches, bench_git_clone);
criterion_main!(benches);
