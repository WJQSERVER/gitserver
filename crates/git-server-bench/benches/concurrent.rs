use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use git_server_bench::fixtures;
use git_server_core::discovery::RepoStore;
use std::net::SocketAddr;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

fn bench_concurrent_clones(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let repo_path = fixtures::medium_repo();
    let (addr, shutdown_tx, handle) = rt.block_on(start_server(repo_path));
    let url = format!("http://{}/repo.git", addr);

    let mut group = c.benchmark_group("concurrent_clones");

    for n in [1u32, 2, 4, 8, 16, 32] {
        group.bench_with_input(BenchmarkId::new("clones", n), &n, |b, &n| {
            b.iter(|| {
                let mut children: Vec<_> = (0..n)
                    .map(|_| {
                        let dest = TempDir::new().unwrap();
                        let child = Command::new("git")
                            .args(["clone", &url, dest.path().to_str().unwrap()])
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .spawn()
                            .unwrap();
                        (child, dest)
                    })
                    .collect();

                for (child, _dest) in &mut children {
                    let status = child.wait().unwrap();
                    assert!(status.success(), "concurrent clone failed");
                }
            });
        });
    }

    group.finish();

    let _ = shutdown_tx.send(());
    rt.block_on(async {
        let _ = handle.await;
    });
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

criterion_group!(benches, bench_concurrent_clones);
criterion_main!(benches);
