mod helpers;

use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::io::Cursor;

use tempfile::TempDir;

use helpers::{TestServer, create_bare_repo_with_commits};

fn make_pktline(data: &str) -> Vec<u8> {
    let len = data.len() + 4;
    format!("{len:04x}{data}").into_bytes()
}

fn decode_pkt_lines(body: &[u8]) -> Vec<String> {
    let mut pos = 0;
    let mut lines = Vec::new();

    while pos + 4 <= body.len() {
        let prefix = std::str::from_utf8(&body[pos..pos + 4]).unwrap();
        pos += 4;
        if prefix == "0000" {
            lines.push("0000".to_string());
            break;
        }
        if prefix == "0001" {
            lines.push("0001".to_string());
            continue;
        }

        let len = usize::from_str_radix(prefix, 16).unwrap();
        let payload = &body[pos..pos + (len - 4)];
        pos += len - 4;
        lines.push(String::from_utf8(payload.to_vec()).unwrap());
    }

    lines
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

fn decode_gzip(body: &[u8]) -> Vec<u8> {
    let mut decoder = flate2::read::GzDecoder::new(body);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded).expect("decode gzip body");
    decoded
}

fn decode_zstd(body: &[u8]) -> Vec<u8> {
    zstd::stream::decode_all(Cursor::new(body)).expect("decode zstd body")
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
async fn upload_pack_v1_multi_ack_returns_continue_for_common_have() {
    let root = TempDir::new().unwrap();
    let bare_path = create_bare_repo_with_commits(root.path(), "multi.git", 2);
    let head_oid = repo_head_oid(&bare_path);
    let server = TestServer::start(root.path()).await;

    let mut request_body = make_pktline(&format!("want {head_oid} multi_ack_detailed side-band-64k ofs-delta\n"));
    request_body.extend_from_slice(b"0000");
    request_body.extend_from_slice(&make_pktline(&format!("have {head_oid}\n")));
    request_body.extend_from_slice(b"0000");

    let response = reqwest::Client::new()
        .post(server.url("multi.git/git-upload-pack"))
        .header("content-type", "application/x-git-upload-pack-request")
        .body(request_body)
        .send()
        .await
        .expect("POST git-upload-pack multi_ack");

    assert_eq!(response.status(), 200);
    let body = response.bytes().await.expect("read multi_ack response");
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains(&format!("ACK {head_oid} common\n")));
    assert!(text.contains("NAK\n"));
    assert!(!text.contains("PACK"), "negotiation round should not send pack yet");

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn git_fetch_works_over_protocol_v1_with_multi_ack() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "multi.git", 1);

    let server = TestServer::start(root.path()).await;
    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("cloned-v1");

    let clone = tokio::task::spawn_blocking({
        let url = server.url("multi.git");
        let clone_path = clone_path.clone();
        move || {
            Command::new("git")
                .args(["-c", "protocol.version=1", "clone", &url, clone_path.to_str().unwrap()])
                .output()
                .expect("git clone protocol v1")
        }
    })
    .await
    .unwrap();
    assert!(
        clone.status.success(),
        "git clone v1 failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&clone.stdout),
        String::from_utf8_lossy(&clone.stderr),
    );

    let bare_path = root.path().join("multi.git");
    let push_dir = TempDir::new().unwrap();
    let push_path = push_dir.path().join("pusher-v1");
    let out = Command::new("git")
        .args([
            "clone",
            bare_path.to_str().unwrap(),
            push_path.to_str().unwrap(),
        ])
        .output()
        .expect("git clone for v1 push");
    assert!(out.status.success(), "git clone failed: {:?}", out);

    for (key, val) in [("user.name", "Test User"), ("user.email", "test@test.com")] {
        let out = Command::new("git")
            .args(["config", key, val])
            .current_dir(&push_path)
            .output()
            .expect("git config");
        assert!(out.status.success(), "git config failed: {:?}", out);
    }

    std::fs::write(push_path.join("v1-extra.txt"), "extra content\n").unwrap();
    let out = Command::new("git")
        .args(["add", "v1-extra.txt"])
        .current_dir(&push_path)
        .output()
        .expect("git add");
    assert!(out.status.success(), "git add failed: {:?}", out);

    let out = Command::new("git")
        .args(["commit", "-m", "second commit v1"])
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

    let fetch = tokio::task::spawn_blocking({
        let clone_path = clone_path.clone();
        move || {
            Command::new("git")
                .args(["-c", "protocol.version=1", "fetch", "origin", "main"])
                .current_dir(&clone_path)
                .output()
                .expect("git fetch protocol v1")
        }
    })
    .await
    .unwrap();

    assert!(
        fetch.status.success(),
        "git fetch v1 failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&fetch.stdout),
        String::from_utf8_lossy(&fetch.stderr),
    );

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
async fn info_refs_advertises_protocol_v2_capabilities() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "v2.git", 1);
    let server = TestServer::start(root.path()).await;

    let response = reqwest::Client::new()
        .get(format!(
            "{}/info/refs?service=git-upload-pack",
            server.url("v2.git")
        ))
        .header("Git-Protocol", "version=2")
        .send()
        .await
        .expect("GET info/refs v2");

    assert_eq!(response.status(), 200);
    let body = response.bytes().await.expect("read v2 advertisement");
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("version 2\n"));
    assert!(text.contains("ls-refs=unborn\n"));
    assert!(text.contains("fetch=shallow wait-for-done\n"));

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn info_refs_supports_gzip_compression() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "compressed.git", 1);
    let server = TestServer::start(root.path()).await;

    let response = reqwest::Client::builder()
        .no_gzip()
        .build()
        .expect("build reqwest client")
        .get(format!(
            "{}/info/refs?service=git-upload-pack",
            server.url("compressed.git")
        ))
        .header("Accept-Encoding", "gzip")
        .send()
        .await
        .expect("GET info/refs gzip");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-encoding").and_then(|v| v.to_str().ok()),
        Some("gzip")
    );

    let body = response.bytes().await.expect("read gzip body");
    let decoded = decode_gzip(&body);
    let text = String::from_utf8_lossy(&decoded);
    assert!(text.contains("refs/heads/main"));

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn info_refs_supports_zstd_compression() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "compressed.git", 1);
    let server = TestServer::start(root.path()).await;

    let response = reqwest::Client::builder()
        .no_zstd()
        .build()
        .expect("build reqwest client")
        .get(format!(
            "{}/info/refs?service=git-upload-pack",
            server.url("compressed.git")
        ))
        .header("Accept-Encoding", "zstd")
        .send()
        .await
        .expect("GET info/refs zstd");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-encoding").and_then(|v| v.to_str().ok()),
        Some("zstd")
    );

    let body = response.bytes().await.expect("read zstd body");
    let decoded = decode_zstd(&body);
    let text = String::from_utf8_lossy(&decoded);
    assert!(text.contains("refs/heads/main"));

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn upload_pack_v2_ls_refs_returns_matching_refs() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "v2.git", 1);
    let server = TestServer::start(root.path()).await;

    let mut body = Vec::new();
    body.extend_from_slice(&make_pktline("command=ls-refs\n"));
    body.extend_from_slice(&make_pktline("object-format=sha1\n"));
    body.extend_from_slice(b"0001");
    body.extend_from_slice(&make_pktline("peel\n"));
    body.extend_from_slice(&make_pktline("symrefs\n"));
    body.extend_from_slice(&make_pktline("ref-prefix refs/heads/\n"));
    body.extend_from_slice(b"0000");

    let response = reqwest::Client::new()
        .post(server.url("v2.git/git-upload-pack"))
        .header("Git-Protocol", "version=2")
        .header("content-type", "application/x-git-upload-pack-request")
        .body(body)
        .send()
        .await
        .expect("POST ls-refs v2");

    assert_eq!(response.status(), 200);
    let body = response.bytes().await.expect("read ls-refs response");
    let lines = decode_pkt_lines(&body);
    assert!(
        lines.iter().any(|line| line.contains("refs/heads/main")),
        "expected refs/heads/main in ls-refs response"
    );

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn git_fetch_works_over_protocol_v2() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "v2.git", 2);
    let server = TestServer::start(root.path()).await;

    let repo_dir = TempDir::new().unwrap();
    let repo_path = repo_dir.path();

    let init = Command::new("git")
        .args(["init", repo_path.to_str().unwrap()])
        .output()
        .expect("git init");
    assert!(init.status.success(), "git init failed: {:?}", init);

    let remote = Command::new("git")
        .args(["remote", "add", "origin", &server.url("v2.git")])
        .current_dir(repo_path)
        .output()
        .expect("git remote add");
    assert!(remote.status.success(), "git remote add failed: {:?}", remote);

    let fetch = tokio::task::spawn_blocking({
        let repo_path = repo_path.to_path_buf();
        let url = server.url("v2.git");
        move || {
            Command::new("git")
                .args(["-c", "protocol.version=2", "fetch", &url, "main"])
                .current_dir(&repo_path)
                .output()
                .expect("git fetch protocol v2")
        }
    })
    .await
    .unwrap();

    assert!(
        fetch.status.success(),
        "git fetch v2 failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&fetch.stdout),
        String::from_utf8_lossy(&fetch.stderr),
    );

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn git_ls_remote_works_over_protocol_v2() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "v2.git", 1);
    let server = TestServer::start(root.path()).await;

    let output = tokio::task::spawn_blocking({
        let url = server.url("v2.git");
        move || {
            Command::new("git")
                .args(["-c", "protocol.version=2", "ls-remote", &url, "HEAD"])
                .output()
                .expect("git ls-remote protocol v2")
        }
    })
    .await
    .unwrap();

    assert!(
        output.status.success(),
        "git ls-remote v2 failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("HEAD"),
        "expected HEAD in ls-remote output"
    );

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn git_clone_works_over_protocol_v2() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "v2.git", 3);
    let server = TestServer::start(root.path()).await;

    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("cloned-v2");

    let output = tokio::task::spawn_blocking({
        let url = server.url("v2.git");
        let clone_path = clone_path.clone();
        move || {
            Command::new("git")
                .args([
                    "-c",
                    "protocol.version=2",
                    "clone",
                    &url,
                    clone_path.to_str().unwrap(),
                ])
                .output()
                .expect("git clone protocol v2")
        }
    })
    .await
    .unwrap();

    assert!(
        output.status.success(),
        "git clone v2 failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&clone_path)
        .output()
        .expect("git log after v2 clone");
    assert!(log.status.success(), "git log failed after v2 clone");
    assert_eq!(String::from_utf8_lossy(&log.stdout).trim().lines().count(), 3);

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn git_shallow_clone_works_over_protocol_v2() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "v2.git", 4);
    let server = TestServer::start(root.path()).await;

    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("shallow-v2");

    let output = tokio::task::spawn_blocking({
        let url = server.url("v2.git");
        let clone_path = clone_path.clone();
        move || {
            Command::new("git")
                .args([
                    "-c",
                    "protocol.version=2",
                    "clone",
                    "--depth=1",
                    &url,
                    clone_path.to_str().unwrap(),
                ])
                .output()
                .expect("git shallow clone protocol v2")
        }
    })
    .await
    .unwrap();

    assert!(
        output.status.success(),
        "git shallow clone v2 failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&clone_path)
        .output()
        .expect("git log after shallow clone");
    assert!(log.status.success(), "git log failed after shallow clone");
    assert_eq!(String::from_utf8_lossy(&log.stdout).trim().lines().count(), 1);

    let shallow = std::fs::read_to_string(clone_path.join(".git/shallow")).expect("read shallow file");
    assert_eq!(shallow.trim().lines().count(), 1, "expected one shallow boundary");

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn git_fetch_deepen_works_over_protocol_v2() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "v2.git", 4);
    let server = TestServer::start(root.path()).await;

    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("shallow-v2");

    let clone = tokio::task::spawn_blocking({
        let url = server.url("v2.git");
        let clone_path = clone_path.clone();
        move || {
            Command::new("git")
                .args([
                    "-c",
                    "protocol.version=2",
                    "clone",
                    "--depth=1",
                    &url,
                    clone_path.to_str().unwrap(),
                ])
                .output()
                .expect("git shallow clone for deepen")
        }
    })
    .await
    .unwrap();
    assert!(clone.status.success(), "initial shallow clone failed: {:?}", clone);

    let deepen = tokio::task::spawn_blocking({
        let clone_path = clone_path.clone();
        move || {
            Command::new("git")
                .args(["-c", "protocol.version=2", "fetch", "--deepen=1", "origin", "main"])
                .current_dir(&clone_path)
                .output()
                .expect("git fetch deepen protocol v2")
        }
    })
    .await
    .unwrap();

    assert!(
        deepen.status.success(),
        "git fetch deepen v2 failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&deepen.stdout),
        String::from_utf8_lossy(&deepen.stderr),
    );

    let log = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&clone_path)
        .output()
        .expect("git log after deepen");
    assert!(log.status.success(), "git log failed after deepen");
    assert_eq!(String::from_utf8_lossy(&log.stdout).trim().lines().count(), 2);

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn upload_pack_v2_fetch_negotiation_returns_acknowledgments() {
    let root = TempDir::new().unwrap();
    let bare_path = create_bare_repo_with_commits(root.path(), "v2.git", 2);
    let head_oid = repo_head_oid(&bare_path);
    let server = TestServer::start(root.path()).await;

    let mut body = Vec::new();
    body.extend_from_slice(&make_pktline("command=fetch\n"));
    body.extend_from_slice(&make_pktline("object-format=sha1\n"));
    body.extend_from_slice(b"0001");
    body.extend_from_slice(&make_pktline("ofs-delta\n"));
    body.extend_from_slice(&make_pktline(&format!("want {head_oid}\n")));
    body.extend_from_slice(&make_pktline(&format!("have {head_oid}\n")));
    body.extend_from_slice(b"0000");

    let response = reqwest::Client::new()
        .post(server.url("v2.git/git-upload-pack"))
        .header("Git-Protocol", "version=2")
        .header("content-type", "application/x-git-upload-pack-request")
        .body(body)
        .send()
        .await
        .expect("POST fetch negotiation v2");

    assert_eq!(response.status(), 200);
    let body = response.bytes().await.expect("read fetch negotiation response");
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("acknowledgments\n"));
    assert!(text.contains(&format!("ACK {head_oid}\n")));

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn upload_pack_v2_deepen_fetch_returns_sections_and_packfile() {
    let root = TempDir::new().unwrap();
    let bare_path = create_bare_repo_with_commits(root.path(), "v2.git", 3);
    let head_oid = repo_head_oid(&bare_path);
    let server = TestServer::start(root.path()).await;

    let mut body = Vec::new();
    body.extend_from_slice(&make_pktline("command=fetch\n"));
    body.extend_from_slice(&make_pktline("object-format=sha1\n"));
    body.extend_from_slice(b"0001");
    body.extend_from_slice(&make_pktline("ofs-delta\n"));
    body.extend_from_slice(&make_pktline(&format!("shallow {head_oid}\n")));
    body.extend_from_slice(&make_pktline("deepen 1\n"));
    body.extend_from_slice(&make_pktline("deepen-relative\n"));
    body.extend_from_slice(&make_pktline(&format!("want {head_oid}\n")));
    body.extend_from_slice(&make_pktline(&format!("have {head_oid}\n")));
    body.extend_from_slice(b"0000");

    let response = reqwest::Client::new()
        .post(server.url("v2.git/git-upload-pack"))
        .header("Git-Protocol", "version=2")
        .header("content-type", "application/x-git-upload-pack-request")
        .body(body)
        .send()
        .await
        .expect("POST deepen fetch v2");

    assert_eq!(response.status(), 200);
    let body = response.bytes().await.expect("read deepen fetch response");
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("acknowledgments\n"));
    assert!(text.contains("ready\n"));
    assert!(text.contains("shallow-info\n"));
    assert!(text.contains("unshallow "));
    assert!(text.contains("packfile\n"));

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
async fn healthz_endpoint_returns_ok() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "alpha.git", 1);

    let server = TestServer::start(root.path()).await;

    let resp = reqwest::get(format!("http://{}/healthz", server.addr))
        .await
        .expect("GET /healthz");
    assert_eq!(resp.status(), 200);

    let json: serde_json::Value = resp.json().await.expect("parse healthz json");
    assert_eq!(json["status"], "ok");

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn info_refs_requires_authentication_when_configured() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "secure.git", 1);

    let server = TestServer::start_with_auth(
        root.path(),
        git_server_http::AuthConfig {
            basic: Some(git_server_http::BasicAuthConfig {
                username: "alice".into(),
                password: "secret".into(),
            }),
            bearer_token: Some("token-123".into()),
        },
    )
    .await;

    let resp = reqwest::get(format!(
        "{}/info/refs?service=git-upload-pack",
        server.url("secure.git")
    ))
    .await
    .expect("GET secure info/refs");
    assert_eq!(resp.status(), 401);

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn info_refs_accepts_bearer_authentication() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "secure.git", 1);

    let server = TestServer::start_with_auth(
        root.path(),
        git_server_http::AuthConfig {
            basic: None,
            bearer_token: Some("token-123".into()),
        },
    )
    .await;

    let resp = reqwest::Client::new()
        .get(format!(
            "{}/info/refs?service=git-upload-pack",
            server.url("secure.git")
        ))
        .header("Authorization", "Bearer token-123")
        .send()
        .await
        .expect("GET secure info/refs with bearer");
    assert_eq!(resp.status(), 200);

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn git_clone_works_with_basic_authentication() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "secure.git", 2);

    let server = TestServer::start_with_auth(
        root.path(),
        git_server_http::AuthConfig {
            basic: Some(git_server_http::BasicAuthConfig {
                username: "alice".into(),
                password: "secret".into(),
            }),
            bearer_token: None,
        },
    )
    .await;

    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("secure-clone");
    let authed_url = format!("http://alice:secret@{}/secure.git", server.addr);

    let output = tokio::task::spawn_blocking({
        let clone_path = clone_path.clone();
        move || {
            Command::new("git")
                .args(["clone", &authed_url, clone_path.to_str().unwrap()])
                .output()
                .expect("git clone with basic auth")
        }
    })
    .await
    .unwrap();

    assert!(
        output.status.success(),
        "git clone with auth failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn git_push_works_over_http_receive_pack() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "push.git", 1);
    let server = TestServer::start(root.path()).await;

    let clone_dir = TempDir::new().unwrap();
    let clone_path = clone_dir.path().join("push-clone");
    let clone = Command::new("git")
        .args(["clone", &server.url("push.git"), clone_path.to_str().unwrap()])
        .output()
        .expect("git clone for push test");
    assert!(clone.status.success(), "git clone failed: {:?}", clone);

    for (key, val) in [("user.name", "Test User"), ("user.email", "test@test.com")] {
        let out = Command::new("git")
            .args(["config", key, val])
            .current_dir(&clone_path)
            .output()
            .expect("git config");
        assert!(out.status.success(), "git config failed: {:?}", out);
    }

    std::fs::write(clone_path.join("push.txt"), "pushed\n").unwrap();
    let out = Command::new("git")
        .args(["add", "push.txt"])
        .current_dir(&clone_path)
        .output()
        .expect("git add push file");
    assert!(out.status.success(), "git add failed: {:?}", out);

    let out = Command::new("git")
        .args(["commit", "-m", "push commit"])
        .current_dir(&clone_path)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .expect("git commit push file");
    assert!(out.status.success(), "git commit failed: {:?}", out);

    let push = Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(&clone_path)
        .output()
        .expect("git push over http");
    assert!(
        push.status.success(),
        "git push failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&push.stdout),
        String::from_utf8_lossy(&push.stderr),
    );

    let remote_head = Command::new("git")
        .args(["rev-parse", "refs/heads/main"])
        .current_dir(root.path().join("push.git"))
        .output()
        .expect("git rev-parse bare head");
    assert!(remote_head.status.success(), "git rev-parse failed: {:?}", remote_head);

    server.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn repository_list_hot_reloads_after_new_repo_appears() {
    let root = TempDir::new().unwrap();
    create_bare_repo_with_commits(root.path(), "alpha.git", 1);

    let server = TestServer::start(root.path()).await;

    let initial = reqwest::get(server.url(""))
        .await
        .expect("GET initial repo list")
        .json::<serde_json::Value>()
        .await
        .expect("parse initial repo list");
    assert_eq!(initial.as_array().unwrap().len(), 1);

    create_bare_repo_with_commits(root.path(), "beta.git", 1);

    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(2);
    let mut saw_beta = false;
    while tokio::time::Instant::now() < deadline {
        let list = reqwest::get(server.url(""))
            .await
            .expect("GET refreshed repo list")
            .json::<serde_json::Value>()
            .await
            .expect("parse refreshed repo list");

        if list
            .as_array()
            .unwrap()
            .iter()
            .any(|repo| repo["name"] == "beta.git")
        {
            saw_beta = true;
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    assert!(saw_beta, "expected beta.git to appear after automatic rescan");

    let info_refs = reqwest::get(format!(
        "{}/info/refs?service=git-upload-pack",
        server.url("beta.git")
    ))
    .await
    .expect("GET beta info/refs");
    assert_eq!(info_refs.status(), 200);

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
