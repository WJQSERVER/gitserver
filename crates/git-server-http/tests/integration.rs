mod helpers;

use std::process::Command;

use tempfile::TempDir;

use helpers::{TestServer, create_bare_repo_with_commits};

#[tokio::test(flavor = "multi_thread")]
async fn clone_bare_repo() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "test.git", 3);

    let server = TestServer::start(root.path()).await;
    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("cloned");

    let url = server.url("test.git");
    let cp = clone_path.clone();
    let out = tokio::task::spawn_blocking(move || {
        Command::new("git")
            .args(["clone", &url, cp.to_str().unwrap()])
            .output()
            .expect("git clone")
    })
    .await
    .unwrap();
    assert!(
        out.status.success(),
        "git clone failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    // Verify we have 3 commits
    let out = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&clone_path)
        .output()
        .expect("git log");
    assert!(out.status.success(), "git log failed");

    let log = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = log.trim().lines().collect();
    assert_eq!(lines.len(), 3, "expected 3 commits, got: {log}");

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn fetch_new_commits() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "test.git", 1);

    let server = TestServer::start(root.path()).await;
    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("cloned");

    // Clone the repo with 1 commit
    let url = server.url("test.git");
    let cp = clone_path.clone();
    let out = tokio::task::spawn_blocking(move || {
        Command::new("git")
            .args(["clone", &url, cp.to_str().unwrap()])
            .output()
            .expect("git clone")
    })
    .await
    .unwrap();
    assert!(
        out.status.success(),
        "git clone failed: stderr={}",
        String::from_utf8_lossy(&out.stderr),
    );

    // Push 1 more commit directly to the bare repo (bypassing the server)
    let bare_path = root.path().join("test.git");
    let push_dir = TempDir::new().unwrap();
    let push_path = push_dir.path().join("pusher");

    let out = Command::new("git")
        .args([
            "clone",
            bare_path.to_str().unwrap(),
            push_path.to_str().unwrap(),
        ])
        .output()
        .expect("git clone for push");
    assert!(out.status.success());

    for (key, val) in [("user.name", "Test User"), ("user.email", "test@test.com")] {
        Command::new("git")
            .args(["config", key, val])
            .current_dir(&push_path)
            .output()
            .expect("git config");
    }

    std::fs::write(push_path.join("extra.txt"), "extra content\n").unwrap();
    Command::new("git")
        .args(["add", "extra.txt"])
        .current_dir(&push_path)
        .output()
        .expect("git add");

    let out = Command::new("git")
        .args(["commit", "-m", "second commit"])
        .current_dir(&push_path)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .expect("git commit");
    assert!(out.status.success(), "git commit failed: {:?}", out);

    let out = Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(&push_path)
        .output()
        .expect("git push");
    assert!(out.status.success(), "git push failed: {:?}", out);

    // Pull from the clone dir (fetching via the HTTP server)
    let cp = clone_path.clone();
    let out = tokio::task::spawn_blocking(move || {
        Command::new("git")
            .args(["pull"])
            .current_dir(&cp)
            .output()
            .expect("git pull")
    })
    .await
    .unwrap();
    assert!(
        out.status.success(),
        "git pull failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    // Verify we have 2 commits
    let out = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&clone_path)
        .output()
        .expect("git log");
    assert!(out.status.success());

    let log = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = log.trim().lines().collect();
    assert_eq!(lines.len(), 2, "expected 2 commits, got: {log}");

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn list_repos_endpoint() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "alpha.git", 1);
    create_bare_repo_with_commits(root.path(), "beta.git", 1);

    let server = TestServer::start(root.path()).await;

    let resp = reqwest::get(server.url("")).await.expect("GET /");
    assert_eq!(resp.status(), 200);

    let json: serde_json::Value = resp.json().await.expect("parse json");
    let arr = json.as_array().expect("should be array");
    assert_eq!(arr.len(), 2, "expected 2 repos, got: {json}");

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn nonexistent_repo_returns_404() {
    let root = TempDir::new().unwrap();
    // Create one repo so the store is non-empty, but we query a different name
    create_bare_repo_with_commits(root.path(), "real.git", 1);

    let server = TestServer::start(root.path()).await;

    let url = format!("{}/info/refs?service=git-upload-pack", server.url("nope.git"));
    let resp = reqwest::get(&url).await.expect("GET nope.git info/refs");
    assert_eq!(resp.status(), 404);

    let json: serde_json::Value = resp.json().await.expect("parse json");
    assert_eq!(json["error"], "not_found");

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn path_traversal_returns_400() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "test.git", 1);

    let server = TestServer::start(root.path()).await;

    // Attempt path traversal via ../
    let url = format!(
        "http://{}/..%2F..%2Fetc%2Fpasswd/info/refs?service=git-upload-pack",
        server.addr
    );
    let resp = reqwest::get(&url).await.expect("GET traversal path");
    let status = resp.status().as_u16();
    assert!(
        status == 400 || status == 404,
        "expected 400 or 404, got {status}"
    );

    server.stop().await;
}
