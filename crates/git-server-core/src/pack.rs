use std::collections::HashSet;
use std::path::Path;

use bytes::Bytes;
use sha1::{Digest, Sha1};
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

use crate::error::{Error, Result};
use crate::pktline;

#[derive(Debug, Clone, Default)]
pub struct UploadPackCapabilities {
    pub ofs_delta: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ShallowRequest {
    pub depth: Option<usize>,
    pub client_shallows: Vec<gix::ObjectId>,
    pub deepen_relative: bool,
}

/// A parsed upload-pack request from a Git client.
pub struct UploadPackRequest {
    pub wants: Vec<gix::ObjectId>,
    pub haves: Vec<gix::ObjectId>,
    pub done: bool,
    pub capabilities: UploadPackCapabilities,
    pub shallow: ShallowRequest,
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
        let mut capabilities = UploadPackCapabilities::default();
        let mut shallow = ShallowRequest::default();
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
            } else if let Some(rest) = line.strip_prefix("deepen ") {
                let depth = rest
                    .parse::<usize>()
                    .map_err(|_| Error::Protocol(format!("invalid deepen value: {rest}")))?;
                shallow.depth = Some(depth);
            } else if line == "deepen-relative" {
                shallow.deepen_relative = true;
            } else if let Some(rest) = line.strip_prefix("shallow ") {
                let oid = gix::ObjectId::from_hex(rest.as_bytes())
                    .map_err(|_| Error::Protocol(format!("invalid OID in shallow: {rest}")))?;
                shallow.client_shallows.push(oid);
            } else if let Some(rest) = line.strip_prefix("want ") {
                let mut parts = rest.split_ascii_whitespace();
                let oid_hex = parts
                    .next()
                    .ok_or_else(|| Error::Protocol("missing OID in want".into()))?;
                let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                    .map_err(|_| Error::Protocol(format!("invalid OID in want: {oid_hex}")))?;
                if wants.is_empty() {
                    for capability in parts {
                        if capability == "ofs-delta" {
                            capabilities.ofs_delta = true;
                        }
                    }
                }
                wants.push(oid);
            } else if let Some(rest) = line.strip_prefix("have ") {
                let oid_hex = rest
                    .split_ascii_whitespace()
                    .next()
                    .ok_or_else(|| Error::Protocol("missing OID in have".into()))?;
                let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                    .map_err(|_| Error::Protocol(format!("invalid OID in have: {oid_hex}")))?;
                haves.push(oid);
            }

            pos += len;
        }

        Ok(Self {
            wants,
            haves,
            done,
            capabilities,
            shallow,
        })
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

fn encode_ofs_delta_base_distance(mut distance: u64) -> Vec<u8> {
    debug_assert!(distance > 0, "offset deltas must point backwards");

    let mut buf = [0u8; 10];
    let mut bytes_written = 1;
    buf[buf.len() - 1] = distance as u8 & 0x7f;

    for out in buf.iter_mut().rev().skip(1) {
        distance >>= 7;
        if distance == 0 {
            break;
        }
        distance -= 1;
        *out = 0x80 | (distance as u8 & 0x7f);
        bytes_written += 1;
    }

    buf[buf.len() - bytes_written..].to_vec()
}

fn encode_delta_size(mut size: usize, out: &mut Vec<u8>) {
    loop {
        let mut byte = (size & 0x7f) as u8;
        size >>= 7;
        if size > 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if size == 0 {
            break;
        }
    }
}

fn encode_delta_copy_instruction(out: &mut Vec<u8>, offset: usize, size: usize) {
    debug_assert!(size > 0 && size <= 0x10000);

    let command_pos = out.len();
    out.push(0x80);
    let mut command = 0x80;

    if offset & 0xff != 0 {
        command |= 0x01;
        out.push(offset as u8);
    }
    if (offset >> 8) & 0xff != 0 {
        command |= 0x02;
        out.push((offset >> 8) as u8);
    }
    if (offset >> 16) & 0xff != 0 {
        command |= 0x04;
        out.push((offset >> 16) as u8);
    }
    if (offset >> 24) & 0xff != 0 {
        command |= 0x08;
        out.push((offset >> 24) as u8);
    }

    if size != 0x10000 {
        if size & 0xff != 0 {
            command |= 0x10;
            out.push(size as u8);
        }
        if (size >> 8) & 0xff != 0 {
            command |= 0x20;
            out.push((size >> 8) as u8);
        }
        if (size >> 16) & 0xff != 0 {
            command |= 0x40;
            out.push((size >> 16) as u8);
        }
    }

    out[command_pos] = command;
}

fn encode_delta_copy(out: &mut Vec<u8>, mut offset: usize, mut size: usize) {
    while size > 0 {
        let chunk = size.min(0x10000);
        encode_delta_copy_instruction(out, offset, chunk);
        offset += chunk;
        size -= chunk;
    }
}

fn encode_delta_insert(out: &mut Vec<u8>, data: &[u8]) {
    for chunk in data.chunks(0x7f) {
        out.push(chunk.len() as u8);
        out.extend_from_slice(chunk);
    }
}

fn encode_blob_delta(base: &[u8], target: &[u8]) -> Option<Vec<u8>> {
    let mut prefix = 0;
    let max_prefix = base.len().min(target.len());
    while prefix < max_prefix && base[prefix] == target[prefix] {
        prefix += 1;
    }

    let max_suffix = base
        .len()
        .saturating_sub(prefix)
        .min(target.len().saturating_sub(prefix));
    let mut suffix = 0;
    while suffix < max_suffix && base[base.len() - 1 - suffix] == target[target.len() - 1 - suffix] {
        suffix += 1;
    }

    if prefix == 0 && suffix == 0 {
        return None;
    }

    let mut delta = Vec::new();
    encode_delta_size(base.len(), &mut delta);
    encode_delta_size(target.len(), &mut delta);

    if prefix > 0 {
        encode_delta_copy(&mut delta, 0, prefix);
    }

    let insert_start = prefix;
    let insert_end = target.len() - suffix;
    encode_delta_insert(&mut delta, &target[insert_start..insert_end]);

    if suffix > 0 {
        encode_delta_copy(&mut delta, base.len() - suffix, suffix);
    }

    Some(delta)
}

fn build_base_entry(kind: gix::object::Kind, data: &[u8]) -> Vec<u8> {
    let type_num = object_type_number(kind);
    let obj_header = encode_pack_object_header(type_num, data.len());
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(data, 6);

    let mut entry = Vec::with_capacity(obj_header.len() + compressed.len());
    entry.extend_from_slice(&obj_header);
    entry.extend_from_slice(&compressed);
    entry
}

fn build_ofs_delta_entry(
    pack_offset: u64,
    base_pack_offset: u64,
    base_data: &[u8],
    target_data: &[u8],
) -> Option<Vec<u8>> {
    let delta = encode_blob_delta(base_data, target_data)?;
    let obj_header = encode_pack_object_header(6, delta.len());
    let base_distance = encode_ofs_delta_base_distance(pack_offset - base_pack_offset);
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&delta, 6);

    let mut entry = Vec::with_capacity(obj_header.len() + base_distance.len() + compressed.len());
    entry.extend_from_slice(&obj_header);
    entry.extend_from_slice(&base_distance);
    entry.extend_from_slice(&compressed);
    Some(entry)
}

struct BlobDeltaBase {
    pack_offset: u64,
    data: Vec<u8>,
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
    let ofs_delta = request.capabilities.ofs_delta;

    let (tx, rx) = tokio::sync::mpsc::channel::<std::result::Result<Bytes, std::io::Error>>(64);

    let handle = tokio::task::spawn_blocking(move || {
        if let Err(e) = generate_pack_sync(&repo_path, &wants, &haves, ofs_delta, &tx) {
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
    ofs_delta: bool,
    tx: &tokio::sync::mpsc::Sender<std::result::Result<Bytes, std::io::Error>>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const MAX_DELTA_BASES: usize = 8;
    const MIN_DELTA_BLOB_SIZE: usize = 1024;

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

    let mut pack_offset = header.len() as u64;
    let mut recent_blob_bases = Vec::<BlobDeltaBase>::new();

    // Each object: read, compress, frame, send
    for oid in &oids {
        let obj = repo.find_object(*oid)?;
        let full_entry = build_base_entry(obj.kind, &obj.data);
        let mut used_delta = false;
        let entry = if ofs_delta && obj.kind == gix::object::Kind::Blob && obj.data.len() >= MIN_DELTA_BLOB_SIZE {
            recent_blob_bases
                .iter()
                .filter(|base| base.data.len() >= MIN_DELTA_BLOB_SIZE)
                .filter_map(|base| {
                    build_ofs_delta_entry(pack_offset, base.pack_offset, &base.data, &obj.data)
                })
                .min_by_key(Vec::len)
                .filter(|delta_entry| delta_entry.len() < full_entry.len())
                .map(|delta_entry| {
                    used_delta = true;
                    delta_entry
                })
                .unwrap_or(full_entry)
        } else {
            full_entry
        };

        hasher.update(&entry);
        send_sideband(tx, &entry)?;

        if obj.kind == gix::object::Kind::Blob && !used_delta && obj.data.len() >= MIN_DELTA_BLOB_SIZE {
            recent_blob_bases.push(BlobDeltaBase {
                pack_offset,
                data: obj.data.to_vec(),
            });
            if recent_blob_bases.len() > MAX_DELTA_BASES {
                recent_blob_bases.remove(0);
            }
        }

        pack_offset += entry.len() as u64;
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
        assert!(!req.capabilities.ofs_delta);
        assert_eq!(req.shallow.depth, None);
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
        assert!(!req.capabilities.ofs_delta);
        assert!(req.shallow.client_shallows.is_empty());
    }

    #[test]
    fn parse_ofs_delta_capability() {
        let hash = "0000000000000000000000000000000000000001";
        let mut body = make_pktline(&format!("want {hash} side-band-64k ofs-delta\n"));
        body.extend_from_slice(b"0009done\n");
        let req = UploadPackRequest::parse(&body).unwrap();
        assert!(req.capabilities.ofs_delta);
    }

    #[test]
    fn parse_shallow_request() {
        let hash = "0000000000000000000000000000000000000001";
        let mut body = make_pktline(&format!("want {hash}\n"));
        body.extend_from_slice(&make_pktline("deepen 2\n"));
        body.extend_from_slice(&make_pktline(&format!("shallow {hash}\n")));
        body.extend_from_slice(&make_pktline("deepen-relative\n"));
        body.extend_from_slice(b"0009done\n");
        let req = UploadPackRequest::parse(&body).unwrap();
        assert_eq!(req.shallow.depth, Some(2));
        assert_eq!(req.shallow.client_shallows, vec![gix::ObjectId::from_hex(hash.as_bytes()).unwrap()]);
        assert!(req.shallow.deepen_relative);
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
            capabilities: UploadPackCapabilities::default(),
            shallow: ShallowRequest::default(),
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
