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

/// An object collected for inclusion in the pack.
struct PackObject {
    kind: gix::object::Kind,
    data: Vec<u8>,
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

/// Wrap data in side-band-64k pkt-line framing (band 1 = pack data).
///
/// Each frame is a pkt-line where the first byte is band 0x01,
/// followed by pack data.
///
/// Git's pkt-line format limits the total line length (including the
/// 4-byte hex length prefix) to 65520 bytes (LARGE_PACKET_MAX).
/// With 4 bytes for the length prefix and 1 byte for the band indicator,
/// the maximum pack data per frame is 65520 - 4 - 1 = 65515 bytes.
fn sideband_pack_data(data: &[u8]) -> Vec<u8> {
    // LARGE_PACKET_MAX (65520) - 4 (pkt-len prefix) - 1 (band byte)
    const MAX_DATA_PER_FRAME: usize = 65515;
    let mut output = Vec::new();

    for chunk in data.chunks(MAX_DATA_PER_FRAME) {
        // pkt-line length = 4 (length prefix) + 1 (band byte) + chunk.len()
        let pkt_len = 4 + 1 + chunk.len();
        output.extend_from_slice(format!("{pkt_len:04x}").as_bytes());
        output.push(0x01); // band 1 = pack data
        output.extend_from_slice(chunk);
    }

    output
}

/// Collect all objects reachable from a tree, recursively.
fn collect_tree_objects(
    repo: &gix::Repository,
    tree_oid: gix::ObjectId,
    seen: &mut HashSet<gix::ObjectId>,
    objects: &mut Vec<PackObject>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !seen.insert(tree_oid) {
        return Ok(());
    }

    let tree_obj = repo.find_object(tree_oid)?;
    let tree_data = tree_obj.data.clone();
    let tree_kind = tree_obj.kind;
    objects.push(PackObject {
        kind: tree_kind,
        data: tree_data.clone(),
    });

    // Parse tree entries using gix's TreeRefIter
    let tree = repo.find_tree(tree_oid)?;
    for entry_result in tree.iter() {
        let entry = entry_result?;
        let entry_oid = entry.inner.oid.to_owned();
        let entry_mode = entry.inner.mode;

        if entry_mode.is_tree() {
            // Recurse into subtrees
            collect_tree_objects(repo, entry_oid, seen, objects)?;
        } else if !seen.contains(&entry_oid) {
            // Blob, symlink, or submodule entry -- collect as-is
            seen.insert(entry_oid);
            if !entry_mode.is_commit() {
                // Skip submodule commits, collect blobs and symlinks
                let obj = repo.find_object(entry_oid)?;
                objects.push(PackObject {
                    kind: obj.kind,
                    data: obj.data.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Build the raw packfile bytes (header + objects + checksum).
fn build_packfile(objects: &[PackObject]) -> Vec<u8> {
    let mut pack = Vec::new();

    // Pack header: "PACK" + version 2 + object count
    pack.extend_from_slice(b"PACK");
    pack.extend_from_slice(&2u32.to_be_bytes()); // version 2
    pack.extend_from_slice(&(objects.len() as u32).to_be_bytes());

    // Each object: header + zlib-compressed data
    for obj in objects {
        let type_num = object_type_number(obj.kind);
        let header = encode_pack_object_header(type_num, obj.data.len());
        pack.extend_from_slice(&header);

        // Compress with zlib (deflate)
        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&obj.data, 6);
        pack.extend_from_slice(&compressed);
    }

    // Footer: SHA-1 checksum of everything
    let mut hasher = Sha1::new();
    hasher.update(&pack);
    let checksum = hasher.finalize();
    pack.extend_from_slice(&checksum);

    pack
}

/// Generate the complete pack response for a Git upload-pack request.
///
/// Returns an `AsyncRead` producing the side-band-64k framed response that
/// can be streamed as the HTTP response body.
pub fn generate_pack(
    repo_path: &Path,
    request: &UploadPackRequest,
) -> Result<impl AsyncRead + Send + Unpin + use<>> {
    // We need owned data for the spawn_blocking closure
    let repo_path = repo_path.to_path_buf();
    let wants: Vec<gix::ObjectId> = request.wants.clone();
    let haves: Vec<gix::ObjectId> = request.haves.clone();

    let (tx, rx) = tokio::sync::mpsc::channel::<std::result::Result<Bytes, std::io::Error>>(16);

    tokio::task::spawn_blocking(move || {
        let result = generate_pack_sync(&repo_path, &wants, &haves);
        match result {
            Ok(data) => {
                // Send data in chunks to avoid holding the entire response in memory
                for chunk in data.chunks(65536) {
                    if tx.blocking_send(Ok(Bytes::copy_from_slice(chunk))).is_err() {
                        return;
                    }
                }
            }
            Err(e) => {
                let _ = tx.blocking_send(Err(std::io::Error::other(e.to_string())));
            }
        }
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Ok(StreamReader::new(stream))
}

/// Synchronous inner implementation of pack generation.
fn generate_pack_sync(
    repo_path: &Path,
    wants: &[gix::ObjectId],
    haves: &[gix::ObjectId],
) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let repo = gix::open(repo_path)?;
    let mut response = Vec::new();

    // Build NAK/ACK line (basic ack mode -- single ACK or NAK)
    response.extend_from_slice(&pktline::encode(b"NAK\n"));

    // Collect objects: walk commits from wants, excluding haves
    let have_set: HashSet<gix::ObjectId> = haves.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut objects = Vec::new();

    // Mark have objects as already seen so we don't include them
    for have in haves {
        seen.insert(*have);
    }

    // Walk from each want
    let walk = repo
        .rev_walk(wants.iter().copied())
        .with_hidden(haves.iter().copied())
        .all()?;

    for info_result in walk {
        let info = info_result?;
        let commit_oid = info.id;

        if have_set.contains(&commit_oid) {
            continue;
        }

        if !seen.insert(commit_oid) {
            continue;
        }

        // Get the commit object data
        let commit_obj = repo.find_object(commit_oid)?;
        let commit_data = commit_obj.data.clone();

        // Get the tree OID from this commit
        let commit = repo.find_commit(commit_oid)?;
        let tree_oid = commit.tree_id()?.detach();

        // Store the commit object
        objects.push(PackObject {
            kind: gix::object::Kind::Commit,
            data: commit_data,
        });

        // Collect tree and all its children
        collect_tree_objects(&repo, tree_oid, &mut seen, &mut objects)?;
    }

    // Build the packfile
    let packfile = build_packfile(&objects);

    // Wrap in side-band-64k framing
    let framed = sideband_pack_data(&packfile);
    response.extend_from_slice(&framed);

    // Send flush to end the side-band stream
    response.extend_from_slice(b"0000");

    Ok(response)
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
