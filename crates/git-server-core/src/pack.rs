use std::collections::HashSet;
use std::path::Path;

use bytes::Bytes;
use sha1::{Digest, Sha1};
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

use crate::error::{Error, Result};
use crate::pktline;

/// A parsed upload-pack request from a Git client.
pub struct UploadPackRequest {
    pub wants: Vec<gix::ObjectId>,
    pub haves: Vec<gix::ObjectId>,
    pub done: bool,
}

impl UploadPackRequest {
    /// Parse a pkt-line encoded upload-pack request body.
    ///
    /// The body contains:
    /// - "want <oid> [capabilities]\n" lines
    /// - flush packet "0000"
    /// - "have <oid>\n" lines (optional)
    /// - "done\n"
    pub fn parse(body: &[u8]) -> Result<Self> {
        let mut wants = Vec::new();
        let mut haves = Vec::new();
        let mut done = false;
        let mut pos = 0;

        while pos < body.len() {
            // Check for flush packet
            if body[pos..].starts_with(b"0000") {
                pos += 4;
                continue;
            }

            // Read 4-byte hex length prefix
            if pos + 4 > body.len() {
                break;
            }
            let len_str = std::str::from_utf8(&body[pos..pos + 4])
                .map_err(|_| Error::Protocol("invalid pkt-line length prefix".into()))?;
            let len = usize::from_str_radix(len_str, 16)
                .map_err(|_| Error::Protocol("invalid pkt-line length".into()))?;

            if len == 0 {
                // flush packet already handled above, but just in case
                pos += 4;
                continue;
            }

            if len < 4 || pos + len > body.len() {
                break;
            }

            let payload = &body[pos + 4..pos + len];
            let line = std::str::from_utf8(payload)
                .map_err(|_| Error::Protocol("invalid UTF-8 in pkt-line".into()))?;
            let line = line.trim_end_matches('\n');

            if line == "done" {
                done = true;
            } else if let Some(rest) = line.strip_prefix("want ") {
                // The OID is the first 40 hex chars; the rest may be capabilities
                let oid_hex = &rest[..40.min(rest.len())];
                let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                    .map_err(|_| Error::Protocol(format!("invalid OID in want: {oid_hex}")))?;
                wants.push(oid);
            } else if let Some(rest) = line.strip_prefix("have ") {
                let oid_hex = &rest[..40.min(rest.len())];
                let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                    .map_err(|_| Error::Protocol(format!("invalid OID in have: {oid_hex}")))?;
                haves.push(oid);
            }

            pos += len;
        }

        Ok(Self { wants, haves, done })
    }
}

/// Encode the variable-length pack object header.
///
/// Format: first byte = MSB continuation + 3-bit type + 4-bit size
/// Subsequent bytes: 7-bit size chunks with MSB continuation
fn encode_pack_object_header(obj_type: u8, size: usize) -> Vec<u8> {
    let mut header = Vec::new();
    let mut byte = (obj_type << 4) | (size as u8 & 0x0f);
    let mut remaining = size >> 4;

    if remaining > 0 {
        byte |= 0x80; // set continuation bit
        header.push(byte);
        while remaining > 0 {
            byte = remaining as u8 & 0x7f;
            remaining >>= 7;
            if remaining > 0 {
                byte |= 0x80;
            }
            header.push(byte);
        }
    } else {
        header.push(byte);
    }

    header
}

/// Map gix object kind to pack type number.
fn object_type_number(kind: gix::object::Kind) -> u8 {
    match kind {
        gix::object::Kind::Commit => 1,
        gix::object::Kind::Tree => 2,
        gix::object::Kind::Blob => 3,
        gix::object::Kind::Tag => 4,
    }
}

/// Send raw bytes through the channel.
fn send(
    tx: &tokio::sync::mpsc::Sender<std::result::Result<Bytes, std::io::Error>>,
    data: &[u8],
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tx.blocking_send(Ok(Bytes::copy_from_slice(data)))
        .map_err(|_| "receiver dropped".into())
}

/// Send pack data through the channel wrapped in side-band-64k framing
/// (band 1 = pack data).
///
/// Respects LARGE_PACKET_MAX: each pkt-line frame carries at most
/// 65520 - 4 (prefix) - 1 (band byte) = 65515 bytes of payload.
fn send_sideband(
    tx: &tokio::sync::mpsc::Sender<std::result::Result<Bytes, std::io::Error>>,
    data: &[u8],
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const MAX_DATA_PER_FRAME: usize = 65515;

    for chunk in data.chunks(MAX_DATA_PER_FRAME) {
        let pkt_len = 4 + 1 + chunk.len();
        let mut frame = Vec::with_capacity(pkt_len);
        frame.extend_from_slice(format!("{pkt_len:04x}").as_bytes());
        frame.push(0x01); // band 1 = pack data
        frame.extend_from_slice(chunk);
        send(tx, &frame)?;
    }

    Ok(())
}

/// Recursively collect tree and blob OIDs reachable from `tree_oid`.
///
/// Uses a single `find_object` call per object and parses raw tree
/// bytes via `TreeRefIter` to avoid a second ODB lookup.
fn collect_tree_oids(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    seen: &mut HashSet<gix::ObjectId>,
    oids: &mut Vec<gix::ObjectId>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !seen.insert(tree_oid) {
        return Ok(());
    }

    let tree_obj = repo.find_object(tree_oid)?;
    let tree_data = tree_obj.data.to_vec();
    oids.push(tree_oid);

    for entry_result in gix::objs::TreeRefIter::from_bytes(&tree_data) {
        let entry = entry_result?;
        let entry_oid = entry.oid.to_owned();
        let entry_mode = entry.mode;

        if entry_mode.is_tree() {
            collect_tree_oids(repo, entry_oid, seen, oids)?;
        } else if seen.insert(entry_oid) && !entry_mode.is_commit() {
            oids.push(entry_oid);
        }
    }

    Ok(())
}

/// Walk commits from `wants` (excluding `haves`) and collect all
/// reachable ObjectIds (commits, trees, blobs).
///
/// Pass 1 of the two-pass streaming approach: only OIDs are stored,
/// not object data.
fn collect_all_oids(
    repo: &gix::Repository,
    wants: &[gix::ObjectId],
    haves: &[gix::ObjectId],
) -> std::result::Result<Vec<gix::ObjectId>, Box<dyn std::error::Error + Send + Sync>> {
    let have_set: HashSet<gix::ObjectId> = haves.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut oids = Vec::new();

    // Mark have objects as already seen so we skip them
    for have in haves {
        seen.insert(*have);
    }

    let walk = repo
        .rev_walk(wants.iter().copied())
        .with_hidden(haves.iter().copied())
        .all()?;

    for info_result in walk {
        let info = info_result?;
        let commit_oid = info.id;

        if have_set.contains(&commit_oid) || !seen.insert(commit_oid) {
            continue;
        }

        // Extract tree OID from raw commit bytes (single ODB read)
        let commit_obj = repo.find_object(commit_oid)?;
        let tree_oid = gix::objs::CommitRefIter::from_bytes(&commit_obj.data).tree_id()?;

        oids.push(commit_oid);

        collect_tree_oids(repo, tree_oid, &mut seen, &mut oids)?;
    }

    Ok(oids)
}

/// Generate the complete pack response for a Git upload-pack request.
///
/// Returns an `AsyncRead` producing the side-band-64k framed response that
/// can be streamed as the HTTP response body.
pub fn generate_pack(
    repo_path: &Path,
    request: &UploadPackRequest,
) -> Result<impl AsyncRead + Send + Unpin + use<>> {
    let repo_path = repo_path.to_path_buf();
    let wants: Vec<gix::ObjectId> = request.wants.clone();
    let haves: Vec<gix::ObjectId> = request.haves.clone();

    let (tx, rx) = tokio::sync::mpsc::channel::<std::result::Result<Bytes, std::io::Error>>(64);

    let handle = tokio::task::spawn_blocking(move || {
        if let Err(e) = generate_pack_sync(&repo_path, &wants, &haves, &tx) {
            let _ = tx.blocking_send(Err(std::io::Error::other(e.to_string())));
        }
    });

    // Log panics from the blocking task without blocking the stream
    tokio::spawn(async move {
        if let Err(e) = handle.await {
            tracing::error!("pack generation task panicked: {e}");
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Ok(StreamReader::new(stream))
}

/// Synchronous two-pass streaming pack generator.
///
/// Pass 1: collect OIDs only (lightweight -- no object data retained).
/// Pass 2: re-read each object, compress, and stream it through `tx`.
fn generate_pack_sync(
    repo_path: &Path,
    wants: &[gix::ObjectId],
    haves: &[gix::ObjectId],
    tx: &tokio::sync::mpsc::Sender<std::result::Result<Bytes, std::io::Error>>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repo = gix::open(repo_path)?;

    // NAK line
    send(tx, &pktline::encode(b"NAK\n"))?;

    // Pass 1: collect OIDs only
    let oids = collect_all_oids(&repo, wants, haves)?;

    // Pass 2: stream each object
    let mut hasher = Sha1::new();

    // Pack header
    let mut header = Vec::with_capacity(12);
    header.extend_from_slice(b"PACK");
    header.extend_from_slice(&2u32.to_be_bytes());
    header.extend_from_slice(&(oids.len() as u32).to_be_bytes());
    hasher.update(&header);
    send_sideband(tx, &header)?;

    // Each object: read, compress, frame, send
    for oid in &oids {
        let obj = repo.find_object(*oid)?;
        let type_num = object_type_number(obj.kind);
        let obj_header = encode_pack_object_header(type_num, obj.data.len());
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&obj.data, 6);

        hasher.update(&obj_header);
        hasher.update(&compressed);

        let mut obj_bytes = Vec::with_capacity(obj_header.len() + compressed.len());
        obj_bytes.extend_from_slice(&obj_header);
        obj_bytes.extend_from_slice(&compressed);
        send_sideband(tx, &obj_bytes)?;
    }

    // SHA-1 checksum over raw pack bytes
    let checksum = hasher.finalize();
    send_sideband(tx, &checksum)?;

    // Flush
    send(tx, b"0000")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use tempfile::TempDir;
    use tokio::io::AsyncReadExt;

    use super::*;

    fn make_pktline(data: &str) -> Vec<u8> {
        let len = data.len() + 4;
        format!("{len:04x}{data}").into_bytes()
    }

    /// Create a bare repo with a single commit on the `main` branch.
    fn create_repo_with_commit(root: &Path) -> PathBuf {
        let bare_path = root.join("test.git");
        let clone_path = root.join("workdir");

        let out = Command::new("git")
            .args(["init", "--bare", bare_path.to_str().unwrap()])
            .output()
            .expect("git init --bare failed");
        assert!(out.status.success(), "git init --bare failed: {:?}", out);

        let out = Command::new("git")
            .args(["symbolic-ref", "HEAD", "refs/heads/main"])
            .current_dir(&bare_path)
            .output()
            .expect("git symbolic-ref failed");
        assert!(out.status.success());

        let out = Command::new("git")
            .args([
                "clone",
                bare_path.to_str().unwrap(),
                clone_path.to_str().unwrap(),
            ])
            .output()
            .expect("git clone failed");
        assert!(out.status.success(), "git clone failed: {:?}", out);

        for (key, val) in [("user.name", "Test User"), ("user.email", "test@test.com")] {
            Command::new("git")
                .args(["config", key, val])
                .current_dir(&clone_path)
                .output()
                .expect("git config failed");
        }

        // Create a file and commit
        std::fs::write(clone_path.join("README.md"), "# Test\n").unwrap();

        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&clone_path)
            .output()
            .expect("git add failed");

        let out = Command::new("git")
            .args(["commit", "-m", "initial commit"])
            .current_dir(&clone_path)
            .env("GIT_AUTHOR_NAME", "Test User")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "Test User")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .output()
            .expect("git commit failed");
        assert!(out.status.success(), "git commit failed: {:?}", out);

        let out = Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&clone_path)
            .output()
            .expect("git push failed");
        assert!(out.status.success(), "git push failed: {:?}", out);

        bare_path
    }

    #[test]
    fn parse_simple_want() {
        let hash = "0000000000000000000000000000000000000001";
        let mut body = make_pktline(&format!("want {hash}\n"));
        body.extend_from_slice(b"00000009done\n");
        let req = UploadPackRequest::parse(&body).unwrap();
        assert_eq!(req.wants.len(), 1);
        assert!(req.haves.is_empty());
        assert!(req.done);
    }

    #[test]
    fn parse_wants_and_haves() {
        let want = "0000000000000000000000000000000000000001";
        let have = "0000000000000000000000000000000000000002";
        let mut body = make_pktline(&format!("want {want}\n"));
        body.extend_from_slice(b"0000");
        body.extend_from_slice(&make_pktline(&format!("have {have}\n")));
        body.extend_from_slice(b"0009done\n");
        let req = UploadPackRequest::parse(&body).unwrap();
        assert_eq!(req.wants.len(), 1);
        assert_eq!(req.haves.len(), 1);
        assert!(req.done);
    }

    #[tokio::test]
    async fn generate_pack_for_clone() {
        let dir = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(dir.path());

        // Get HEAD OID
        let repo = gix::open(&repo_path).unwrap();
        let head_oid = repo.head_id().unwrap().detach();
        drop(repo);

        let request = UploadPackRequest {
            wants: vec![head_oid],
            haves: vec![],
            done: true,
        };

        let mut reader = generate_pack(&repo_path, &request).unwrap();
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();

        let response = String::from_utf8_lossy(&buf);
        assert!(
            response.contains("NAK"),
            "response should contain NAK: {response:?}"
        );

        // Find PACK signature in the binary response
        let pack_found = buf.windows(4).any(|window| window == b"PACK");
        assert!(pack_found, "response should contain PACK signature");
    }
}
