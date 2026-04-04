mod helpers;

use std::path::Path;
use std::process::Command;
use std::io::Cursor;

use tempfile::TempDir;

use helpers::{TestServer, create_bare_repo_with_commits};

fn make_pktline(data: &str) -> Vec<u8> {
    let len = data.len() + 4;
    format!("{len:04x}{data}").into_bytes()
}

fn repo_head_oid(repo_path: &Path) -> String {
    let out = Command::new("git")
        .args(["rev-parse", "refs/heads/main"])
        .current_dir(repo_path)
        .output()
        .expect("git rev-parse");
    assert!(
        out.status.success(),
        "git rev-parse failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    String::from_utf8(out.stdout)
        .expect("head oid should be valid utf-8")
        .trim()
        .to_owned()
}

fn add_large_commit(repo_path: &Path) {
    let work_dir = TempDir::new().unwrap();
    let work_path = work_dir.path().join("pusher");

    let out = Command::new("git")
        .args([
            "clone",
            repo_path.to_str().unwrap(),
            work_path.to_str().unwrap(),
        ])
        .output()
        .expect("git clone for large commit");
    assert!(out.status.success(), "git clone failed: {:?}", out);

    for (key, val) in [("user.name", "Test User"), ("user.email", "test@test.com")] {
        let out = Command::new("git")
            .args(["config", key, val])
            .current_dir(&work_path)
            .output()
            .expect("git config");
        assert!(out.status.success(), "git config failed: {:?}", out);
    }

    let payload: Vec<u8> = (0..(2 * 1024 * 1024)).map(|i| (i % 251) as u8).collect();
    std::fs::write(work_path.join("large.bin"), payload).unwrap();

    let out = Command::new("git")
        .args(["add", "large.bin"])
        .current_dir(&work_path)
        .output()
        .expect("git add");
    assert!(out.status.success(), "git add failed: {:?}", out);

    let out = Command::new("git")
        .args(["commit", "-m", "large payload"])
        .current_dir(&work_path)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .expect("git commit");
    assert!(out.status.success(), "git commit failed: {:?}", out);

    let out = Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(&work_path)
        .output()
        .expect("git push");
    assert!(out.status.success(), "git push failed: {:?}", out);
}

fn strip_sideband_pack(body: &[u8]) -> Vec<u8> {
    let mut pos = 0;
    let mut pack = Vec::new();

    while pos + 4 <= body.len() {
        let len = usize::from_str_radix(std::str::from_utf8(&body[pos..pos + 4]).unwrap(), 16)
            .expect("pkt-line length");
        pos += 4;

        if len == 0 {
            break;
        }

        let payload_len = len - 4;
        let payload = &body[pos..pos + payload_len];
        pos += payload_len;

        if payload == b"NAK\n" {
            continue;
        }

        if let Some(pack_payload) = payload.strip_prefix(&[0x01]) {
            pack.extend_from_slice(pack_payload);
        }
    }

    pack
}

fn pack_contains_ofs_delta(pack: &[u8]) -> bool {
    let reader = Cursor::new(pack);
    let mut iter = gix_pack::data::input::BytesToEntriesIter::new_from_header(
        std::io::BufReader::new(reader),
        gix_pack::data::input::Mode::Verify,
        gix_pack::data::input::EntryDataMode::Ignore,
        gix::hash::Kind::Sha1,
    )
    .expect("parse pack header");

    iter.any(|entry| {
        matches!(
            entry.expect("parse pack entry").header,
            gix_pack::data::entry::Header::OfsDelta { .. }
        )
    })
}

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
async fn upload_pack_streams_large_responses() {
    let root = TempDir::new().unwrap();
    let bare_path = create_bare_repo_with_commits(root.path(), "stream.git", 1);
    add_large_commit(&bare_path);
    let head_oid = repo_head_oid(&bare_path);

    let server = TestServer::start(root.path()).await;

    let mut request_body = make_pktline(&format!("want {head_oid}\n"));
    request_body.extend_from_slice(b"0000");
    request_body.extend_from_slice(b"0009done\n");

    let client = reqwest::Client::new();
    let mut response = client
        .post(server.url("stream.git/git-upload-pack"))
        .header("content-type", "application/x-git-upload-pack-request")
        .body(request_body)
        .send()
        .await
        .expect("POST git-upload-pack");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.content_length(),
        None,
        "streamed upload-pack responses should not advertise a fixed content length"
    );

    let first_chunk = response
        .chunk()
        .await
        .expect("read first response chunk")
        .expect("expected first response chunk");
    assert!(
        first_chunk.starts_with(b"0008NAK\n"),
        "streamed response should start with upload-pack preamble"
    );

    let mut body = first_chunk.to_vec();
    let mut chunk_count = 1;
    while let Some(chunk) = response.chunk().await.expect("read response chunk") {
        chunk_count += 1;
        body.extend_from_slice(&chunk);
    }

    assert!(
        chunk_count > 1,
        "large pack responses should be readable incrementally"
    );
    assert!(
        body.windows(4).any(|window| window == b"PACK"),
        "streamed response should contain pack data"
    );

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn upload_pack_uses_ofs_delta_when_requested() {
    let root = TempDir::new().unwrap();
    let bare_path = create_bare_repo_with_commits(root.path(), "delta.git", 1);

    let work_dir = TempDir::new().unwrap();
    let work_path = work_dir.path().join("pusher");
    let out = Command::new("git")
        .args([
            "clone",
            bare_path.to_str().unwrap(),
            work_path.to_str().unwrap(),
        ])
        .output()
        .expect("git clone for delta setup");
    assert!(out.status.success(), "git clone failed: {:?}", out);

    for (key, val) in [("user.name", "Test User"), ("user.email", "test@test.com")] {
        let out = Command::new("git")
            .args(["config", key, val])
            .current_dir(&work_path)
            .output()
            .expect("git config");
        assert!(out.status.success(), "git config failed: {:?}", out);
    }

    let base = vec![b'a'; 8192];
    std::fs::write(work_path.join("delta.txt"), &base).unwrap();
    let out = Command::new("git")
        .args(["add", "delta.txt"])
        .current_dir(&work_path)
        .output()
        .expect("git add delta base");
    assert!(out.status.success(), "git add failed: {:?}", out);
    let out = Command::new("git")
        .args(["commit", "-m", "delta base"])
        .current_dir(&work_path)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .expect("git commit delta base");
    assert!(out.status.success(), "git commit failed: {:?}", out);

    let mut changed = base.clone();
    changed[4096..4120].copy_from_slice(b"ofs-delta-verification!!");
    std::fs::write(work_path.join("delta.txt"), &changed).unwrap();
    let out = Command::new("git")
        .args(["add", "delta.txt"])
        .current_dir(&work_path)
        .output()
        .expect("git add delta target");
    assert!(out.status.success(), "git add failed: {:?}", out);
    let out = Command::new("git")
        .args(["commit", "-m", "delta target"])
        .current_dir(&work_path)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .expect("git commit delta target");
    assert!(out.status.success(), "git commit failed: {:?}", out);

    let out = Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(&work_path)
        .output()
        .expect("git push delta history");
    assert!(out.status.success(), "git push failed: {:?}", out);

    let head_oid = repo_head_oid(&bare_path);
    let server = TestServer::start(root.path()).await;

    let mut request_body =
        make_pktline(&format!("want {head_oid} side-band-64k ofs-delta\n"));
    request_body.extend_from_slice(b"0000");
    request_body.extend_from_slice(b"0009done\n");

    let response = reqwest::Client::new()
        .post(server.url("delta.git/git-upload-pack"))
        .header("content-type", "application/x-git-upload-pack-request")
        .body(request_body)
        .send()
        .await
        .expect("POST git-upload-pack for ofs-delta");

    assert_eq!(response.status(), 200);
    let body = response.bytes().await.expect("read response body");
    let pack = strip_sideband_pack(&body);
    assert!(pack.starts_with(b"PACK"), "expected a pack payload");
    assert!(
        pack_contains_ofs_delta(&pack),
        "expected OFS_DELTA entry when client requests ofs-delta"
    );

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

    let url = format!(
        "{}/info/refs?service=git-upload-pack",
        server.url("nope.git")
    );
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
