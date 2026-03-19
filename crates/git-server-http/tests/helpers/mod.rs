use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use git_server_core::discovery::RepoStore;

/// A test HTTP server that serves git repositories via the git-server-http router.
pub struct TestServer {
    pub addr: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl TestServer {
    /// Start a test server serving repositories discovered under `root`.
    pub async fn start(root: &Path) -> Self {
        let store = RepoStore::discover(root.to_path_buf(), 0).expect("discover repos");
        let router = git_server_http::router(store);

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let handle = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("server error");
        });

        Self {
            addr,
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
        }
    }

    /// Return the full URL for a given repository name (e.g. "test.git").
    pub fn url(&self, repo: &str) -> String {
        format!("http://{}/{}", self.addr, repo)
    }

    /// Gracefully shut down the server.
    pub async fn stop(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

/// Create a bare git repository with a given number of commits.
///
/// Each commit adds a file named `fileN.txt` where N is the commit index (0-based).
/// Returns the path to the bare repository.
pub fn create_bare_repo_with_commits(root: &Path, name: &str, commit_count: usize) -> PathBuf {
    let bare_path = root.join(name);
    let work_dir = TempDir::new().expect("create temp work dir");
    let work_path = work_dir.path();

    // Create bare repository
    let out = Command::new("git")
        .args(["init", "--bare", bare_path.to_str().unwrap()])
        .output()
        .expect("git init --bare");
    assert!(out.status.success(), "git init --bare failed: {:?}", out);

    // Set default branch to main
    let out = Command::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(&bare_path)
        .output()
        .expect("git symbolic-ref");
    assert!(out.status.success(), "git symbolic-ref failed: {:?}", out);

    // Clone into working directory
    let out = Command::new("git")
        .args([
            "clone",
            bare_path.to_str().unwrap(),
            work_path.to_str().unwrap(),
        ])
        .output()
        .expect("git clone");
    assert!(out.status.success(), "git clone failed: {:?}", out);

    // Configure user
    for (key, val) in [("user.name", "Test User"), ("user.email", "test@test.com")] {
        let out = Command::new("git")
            .args(["config", key, val])
            .current_dir(work_path)
            .output()
            .expect("git config");
        assert!(out.status.success(), "git config failed: {:?}", out);
    }

    // Create commits
    for i in 0..commit_count {
        let filename = format!("file{i}.txt");
        let content = format!("content of file {i}\n");
        std::fs::write(work_path.join(&filename), content).expect("write file");

        let out = Command::new("git")
            .args(["add", &filename])
            .current_dir(work_path)
            .output()
            .expect("git add");
        assert!(out.status.success(), "git add failed: {:?}", out);

        let msg = format!("commit {i}");
        let out = Command::new("git")
            .args(["commit", "-m", &msg])
            .current_dir(work_path)
            .env("GIT_AUTHOR_NAME", "Test User")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "Test User")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .output()
            .expect("git commit");
        assert!(out.status.success(), "git commit failed: {:?}", out);
    }

    // Push to bare repo
    if commit_count > 0 {
        let out = Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(work_path)
            .output()
            .expect("git push");
        assert!(out.status.success(), "git push failed: {:?}", out);
    }

    bare_path
}
