mod helpers;

use helpers::{TestServer, create_bare_repo_with_commits};
use std::process::Command;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_clones_same_repo() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "shared.git", 5);

    let server = TestServer::start(root.path()).await;

    let mut handles = Vec::new();
    for i in 0..10 {
        let url = server.url("shared.git");
        handles.push(tokio::task::spawn_blocking(move || {
            let clone_dir = TempDir::new().unwrap();
            let output = Command::new("git")
                .args(["clone", &url, clone_dir.path().to_str().unwrap()])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "clone {i} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_clones_different_repos() {
    let root = TempDir::new().unwrap();
    for i in 0..5 {
        create_bare_repo_with_commits(root.path(), &format!("repo{i}.git"), 3);
    }

    let server = TestServer::start(root.path()).await;

    let mut handles = Vec::new();
    for i in 0..5 {
        let url = server.url(&format!("repo{i}.git"));
        handles.push(tokio::task::spawn_blocking(move || {
            let clone_dir = TempDir::new().unwrap();
            let output = Command::new("git")
                .args(["clone", &url, clone_dir.path().to_str().unwrap()])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "clone repo{i} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    server.stop().await;
}
