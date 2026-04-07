// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2026 WJQSERVER

use std::collections::BTreeSet;
use std::collections::HashSet;
use std::io::Cursor;
use std::path::Path;

use crate::error::{Error, Result};
use crate::pack::UploadPackRequest;
use crate::pktline;

const CAPABILITIES: &[&str] = &[
    "ls-refs=unborn",
    "fetch=shallow wait-for-done",
    "object-format=sha1",
];

pub enum Command {
    LsRefs(LsRefsRequest),
    Fetch(FetchRequest),
}

#[derive(Default)]
pub struct LsRefsRequest {
    pub peel: bool,
    pub symrefs: bool,
    pub unborn: bool,
    pub ref_prefixes: Vec<String>,
}

pub struct FetchRequest {
    pub upload_request: UploadPackRequest,
}

pub struct ShallowUpdate {
    pub shallow: Vec<gix::ObjectId>,
    pub unshallow: Vec<gix::ObjectId>,
}

pub fn advertise_capabilities() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&pktline::encode_comment("service=git-upload-pack"));
    out.extend_from_slice(pktline::flush());
    out.extend_from_slice(&pktline::encode(b"version 2\n"));
    for capability in CAPABILITIES {
        out.extend_from_slice(&pktline::encode(format!("{capability}\n").as_bytes()));
    }
    out.extend_from_slice(pktline::flush());
    out
}

pub fn parse_command_request(body: &[u8]) -> Result<Command> {
    let lines = decode_pkt_lines(body)?;
    let mut iter = lines.into_iter();

    let command = iter
        .next()
        .ok_or_else(|| Error::Protocol("missing protocol v2 command".into()))?;
    let command = command
        .strip_prefix("command=")
        .ok_or_else(|| Error::Protocol("invalid protocol v2 command line".into()))?;

    let mut args = Vec::new();
    let mut saw_delim = false;
    for line in iter {
        if line.is_empty() {
            continue;
        }
        if line == "0001" {
            saw_delim = true;
            continue;
        }
        if saw_delim {
            args.push(line);
        }
    }

    match command {
        "ls-refs" => parse_ls_refs(args),
        "fetch" => parse_fetch(args),
        _ => Err(Error::Protocol(format!(
            "unsupported protocol v2 command: {command}"
        ))),
    }
}

pub fn ls_refs(repo_path: &Path, request: &LsRefsRequest) -> Result<Vec<u8>> {
    let repo = gix::open(repo_path)?;
    let mut refs = BTreeSet::new();

    if let Ok(mut head) = repo.head() {
        if let Some(id) = head
            .try_peel_to_id()
            .map_err(|e| Error::Protocol(e.to_string()))?
        {
            let mut line = format!("{} HEAD", id.detach());
            if request.symrefs
                && let Some(target) = head.referent_name()
            {
                line.push_str(&format!(" symref-target:{}", target.as_bstr()));
            }
            refs.insert(line);
        } else if request.unborn
            && let Some(target) = head.referent_name()
        {
            refs.insert(format!("unborn HEAD symref-target:{}", target.as_bstr()));
        }
    }

    if let Ok(platform) = repo.references()
        && let Ok(iter) = platform.all()
    {
        for mut reference in iter.flatten() {
            let name = reference.name().as_bstr().to_string();
            if !request.ref_prefixes.is_empty()
                && !request
                    .ref_prefixes
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
            {
                continue;
            }

            let mut line = match reference.try_id() {
                Some(id) => format!("{} {name}", id.detach()),
                None => match reference.peel_to_id() {
                    Ok(id) => format!("{} {name}", id.detach()),
                    Err(_) => continue,
                },
            };

            if request.symrefs
                && let Some(target) = reference.target().try_name()
            {
                line.push_str(&format!(" symref-target:{}", target.as_bstr()));
            }

            if request.peel
                && let Ok(peeled) = reference.peel_to_id()
            {
                line.push_str(&format!(" peeled:{}", peeled.detach()));
            }

            refs.insert(line);
        }
    }

    let mut out = Vec::new();
    for line in refs {
        out.extend_from_slice(&pktline::encode(format!("{line}\n").as_bytes()));
    }
    out.extend_from_slice(pktline::flush());
    Ok(out)
}

pub fn encode_fetch_pack_response(pack_bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&pktline::encode(b"packfile\n"));

    let mut pos = 0;
    while pos + 4 <= pack_bytes.len() {
        let len_str = match std::str::from_utf8(&pack_bytes[pos..pos + 4]) {
            Ok(v) => v,
            Err(_) => break,
        };
        pos += 4;

        if len_str == "0000" {
            out.extend_from_slice(b"0000");
            break;
        }

        let len = match usize::from_str_radix(len_str, 16) {
            Ok(v) if v >= 4 && pos + (v - 4) <= pack_bytes.len() => v,
            _ => break,
        };

        let frame = &pack_bytes[pos - 4..pos + (len - 4)];
        let payload = &pack_bytes[pos..pos + (len - 4)];
        pos += len - 4;

        if payload.starts_with(&[0x01])
            || payload.starts_with(&[0x02])
            || payload.starts_with(&[0x03])
        {
            out.extend_from_slice(frame);
        }
    }

    out
}

pub struct PrefixThenReader<R> {
    prefix: Cursor<Vec<u8>>,
    reader: R,
}

impl<R> PrefixThenReader<R> {
    pub fn new(prefix: Vec<u8>, reader: R) -> Self {
        Self {
            prefix: Cursor::new(prefix),
            reader,
        }
    }
}

pub struct PackSectionReader<R> {
    reader: R,
    buf: Vec<u8>,
    out: Cursor<Vec<u8>>,
    finished: bool,
}

impl<R> PackSectionReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: Vec::new(),
            out: Cursor::new(Vec::new()),
            finished: false,
        }
    }
}

impl<R: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for PackSectionReader<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            if (self.out.position() as usize) < self.out.get_ref().len() {
                let remaining = &self.out.get_ref()[self.out.position() as usize..];
                let to_copy = remaining.len().min(buf.remaining());
                buf.put_slice(&remaining[..to_copy]);
                let next = self.out.position() + to_copy as u64;
                self.out.set_position(next);
                return std::task::Poll::Ready(Ok(()));
            }

            if self.finished {
                return std::task::Poll::Ready(Ok(()));
            }

            let mut frame_buf = [0u8; 8192];
            let mut read_buf = tokio::io::ReadBuf::new(&mut frame_buf);
            match std::pin::Pin::new(&mut self.reader).poll_read(cx, &mut read_buf) {
                std::task::Poll::Pending => return std::task::Poll::Pending,
                std::task::Poll::Ready(Err(err)) => return std::task::Poll::Ready(Err(err)),
                std::task::Poll::Ready(Ok(())) => {
                    let filled = read_buf.filled();
                    if filled.is_empty() {
                        self.finished = true;
                        return std::task::Poll::Ready(Ok(()));
                    }
                    self.buf.extend_from_slice(filled);
                }
            }

            let mut emitted = Vec::new();
            loop {
                if self.buf.len() < 4 {
                    break;
                }
                let len_str = match std::str::from_utf8(&self.buf[..4]) {
                    Ok(v) => v,
                    Err(_) => {
                        self.finished = true;
                        return std::task::Poll::Ready(Err(std::io::Error::other(
                            "invalid pkt-line prefix in pack response",
                        )));
                    }
                };

                if len_str == "0000" {
                    emitted.extend_from_slice(b"0000");
                    self.buf.drain(..4);
                    self.finished = true;
                    break;
                }

                let len = match usize::from_str_radix(len_str, 16) {
                    Ok(v) if v >= 4 => v,
                    _ => {
                        self.finished = true;
                        return std::task::Poll::Ready(Err(std::io::Error::other(
                            "invalid pkt-line length in pack response",
                        )));
                    }
                };

                if self.buf.len() < len {
                    break;
                }

                let frame = self.buf[..len].to_vec();
                let payload = &frame[4..];
                if payload.starts_with(&[0x01])
                    || payload.starts_with(&[0x02])
                    || payload.starts_with(&[0x03])
                {
                    emitted.extend_from_slice(&frame);
                }
                self.buf.drain(..len);
            }

            if !emitted.is_empty() {
                self.out = Cursor::new(emitted);
                self.out.set_position(0);
            }
        }
    }
}

impl<R: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for PrefixThenReader<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if (self.prefix.position() as usize) < self.prefix.get_ref().len() {
            let remaining = &self.prefix.get_ref()[self.prefix.position() as usize..];
            let to_copy = remaining.len().min(buf.remaining());
            buf.put_slice(&remaining[..to_copy]);
            let next = self.prefix.position() + to_copy as u64;
            self.prefix.set_position(next);
            return std::task::Poll::Ready(Ok(()));
        }

        std::pin::Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}

pub fn encode_fetch_ready_and_acknowledgments(common: &[gix::ObjectId]) -> Vec<u8> {
    let mut out = encode_fetch_acknowledgments(common);
    if !common.is_empty() {
        out.truncate(out.len() - 4);
        out.extend_from_slice(&pktline::encode(b"ready\n"));
        out.extend_from_slice(b"0001");
    }
    out
}

pub fn encode_fetch_acknowledgments(common: &[gix::ObjectId]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&pktline::encode(b"acknowledgments\n"));

    if common.is_empty() {
        out.extend_from_slice(&pktline::encode(b"NAK\n"));
    } else {
        for oid in common {
            out.extend_from_slice(&pktline::encode(format!("ACK {oid}\n").as_bytes()));
        }
    }

    out.extend_from_slice(pktline::flush());
    out
}

pub fn encode_shallow_info(update: &ShallowUpdate) -> Vec<u8> {
    let mut out = Vec::new();
    if update.shallow.is_empty() && update.unshallow.is_empty() {
        return out;
    }

    out.extend_from_slice(&pktline::encode(b"shallow-info\n"));
    for oid in &update.shallow {
        out.extend_from_slice(&pktline::encode(format!("shallow {oid}\n").as_bytes()));
    }
    for oid in &update.unshallow {
        out.extend_from_slice(&pktline::encode(format!("unshallow {oid}\n").as_bytes()));
    }
    out.extend_from_slice(b"0001");
    out
}

pub fn common_haves(repo_path: &Path, request: &FetchRequest) -> Result<Vec<gix::ObjectId>> {
    let repo = gix::open(repo_path)?;
    let want_set: HashSet<gix::ObjectId> =
        collect_want_closure(&repo, &request.upload_request.wants)?
            .into_iter()
            .collect();

    Ok(request
        .upload_request
        .haves
        .iter()
        .copied()
        .filter(|oid| want_set.contains(oid))
        .collect())
}

pub fn apply_shallow_boundaries(
    repo_path: &Path,
    request: &mut FetchRequest,
) -> Result<ShallowUpdate> {
    let Some(depth) = request.upload_request.shallow.depth else {
        return Ok(ShallowUpdate {
            shallow: Vec::new(),
            unshallow: Vec::new(),
        });
    };

    let repo = gix::open(repo_path)?;
    let previous_shallows = request.upload_request.shallow.client_shallows.clone();
    let state = collect_depth_limited_commits(&repo, &request.upload_request, depth)?;

    request.upload_request.object_ids = Some(state.included_objects.clone());
    request
        .upload_request
        .haves
        .extend(previous_shallows.iter().copied());

    let next_shallows: HashSet<_> = state.shallow_boundary.iter().copied().collect();
    let prev_shallows: HashSet<_> = previous_shallows.iter().copied().collect();

    Ok(ShallowUpdate {
        shallow: state
            .shallow_boundary
            .iter()
            .copied()
            .filter(|oid| !prev_shallows.contains(oid))
            .collect(),
        unshallow: previous_shallows
            .into_iter()
            .filter(|oid| !next_shallows.contains(oid))
            .collect(),
    })
}

struct DepthState {
    included_objects: Vec<gix::ObjectId>,
    shallow_boundary: Vec<gix::ObjectId>,
}

fn parse_ls_refs(args: Vec<String>) -> Result<Command> {
    let mut request = LsRefsRequest::default();

    for arg in args {
        match arg.as_str() {
            "peel" => request.peel = true,
            "symrefs" => request.symrefs = true,
            "unborn" => request.unborn = true,
            _ => {
                if let Some(prefix) = arg.strip_prefix("ref-prefix ") {
                    request.ref_prefixes.push(prefix.to_owned());
                } else {
                    return Err(Error::Protocol(format!(
                        "unsupported ls-refs argument: {arg}"
                    )));
                }
            }
        }
    }

    Ok(Command::LsRefs(request))
}

fn parse_fetch(args: Vec<String>) -> Result<Command> {
    let mut wants = Vec::new();
    let mut haves = Vec::new();
    let mut done = false;
    let mut capabilities = crate::pack::UploadPackCapabilities::default();
    let mut shallow = crate::pack::ShallowRequest::default();

    for arg in args {
        if arg == "done" {
            done = true;
        } else if arg == "ofs-delta" {
            capabilities.ofs_delta = true;
        } else if arg == "deepen-relative" {
            shallow.deepen_relative = true;
        } else if arg == "thin-pack"
            || arg == "no-progress"
            || arg == "include-tag"
            || arg == "wait-for-done"
        {
            continue;
        } else if let Some(depth) = arg.strip_prefix("deepen ") {
            shallow.depth = Some(
                depth
                    .parse::<usize>()
                    .map_err(|_| Error::Protocol(format!("invalid deepen value: {depth}")))?,
            );
        } else if let Some(oid_hex) = arg.strip_prefix("shallow ") {
            let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                .map_err(|_| Error::Protocol(format!("invalid OID in shallow: {oid_hex}")))?;
            shallow.client_shallows.push(oid);
        } else if let Some(oid_hex) = arg.strip_prefix("want ") {
            let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                .map_err(|_| Error::Protocol(format!("invalid OID in want: {oid_hex}")))?;
            wants.push(oid);
        } else if let Some(oid_hex) = arg.strip_prefix("have ") {
            let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                .map_err(|_| Error::Protocol(format!("invalid OID in have: {oid_hex}")))?;
            haves.push(oid);
        } else {
            return Err(Error::Protocol(format!(
                "unsupported fetch argument: {arg}"
            )));
        }
    }

    Ok(Command::Fetch(FetchRequest {
        upload_request: UploadPackRequest {
            wants,
            haves,
            done,
            capabilities,
            shallow,
            object_ids: None,
        },
    }))
}

fn decode_pkt_lines(body: &[u8]) -> Result<Vec<String>> {
    let mut pos = 0;
    let mut out = Vec::new();

    while pos < body.len() {
        if pos + 4 > body.len() {
            return Err(Error::Protocol("truncated pkt-line prefix".into()));
        }

        let len_str = std::str::from_utf8(&body[pos..pos + 4])
            .map_err(|_| Error::Protocol("invalid pkt-line length prefix".into()))?;
        pos += 4;

        if len_str == "0000" {
            break;
        }
        if len_str == "0001" {
            out.push("0001".to_string());
            continue;
        }

        let len = usize::from_str_radix(len_str, 16)
            .map_err(|_| Error::Protocol("invalid pkt-line length".into()))?;
        if len < 4 || pos + (len - 4) > body.len() {
            return Err(Error::Protocol("invalid pkt-line frame length".into()));
        }

        let payload = &body[pos..pos + (len - 4)];
        pos += len - 4;
        let line = std::str::from_utf8(payload)
            .map_err(|_| Error::Protocol("invalid UTF-8 in pkt-line".into()))?;
        out.push(line.trim_end_matches('\n').to_owned());
    }

    Ok(out)
}

fn collect_want_closure(
    repo: &gix::Repository,
    wants: &[gix::ObjectId],
) -> Result<Vec<gix::ObjectId>> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    let walk = repo
        .rev_walk(wants.iter().copied())
        .all()
        .map_err(|e| Error::Protocol(e.to_string()))?;
    for info_result in walk {
        let info = info_result.map_err(|e| Error::Protocol(e.to_string()))?;
        let commit_oid = info.id;
        if !seen.insert(commit_oid) {
            continue;
        }
        out.push(commit_oid);
    }

    Ok(out)
}

fn collect_depth_limited_commits(
    repo: &gix::Repository,
    request: &crate::pack::UploadPackRequest,
    depth: usize,
) -> Result<DepthState> {
    use std::collections::{HashSet, VecDeque};

    let mut queue = VecDeque::new();
    let mut seen = HashSet::new();
    let mut included_commits = Vec::new();
    let mut included_objects = Vec::new();
    let mut shallow_boundary = Vec::new();

    let base_depth = if request.shallow.deepen_relative {
        1usize
    } else {
        0
    };
    let limit = base_depth + depth;

    for want in &request.wants {
        queue.push_back((*want, 1usize));
    }

    while let Some((commit_oid, current_depth)) = queue.pop_front() {
        if !seen.insert(commit_oid) {
            continue;
        }
        included_commits.push(commit_oid);
        included_objects.push(commit_oid);

        let commit_obj = repo
            .find_object(commit_oid)
            .map_err(|e| Error::Protocol(e.to_string()))?;
        let tree_oid = gix::objs::CommitRefIter::from_bytes(&commit_obj.data)
            .tree_id()
            .map_err(|e| Error::Protocol(e.to_string()))?;
        collect_tree_oids(repo, tree_oid, &mut seen, &mut included_objects)?;
        let parents: Vec<_> = gix::objs::CommitRefIter::from_bytes(&commit_obj.data)
            .parent_ids()
            .collect();

        if current_depth >= limit || parents.is_empty() {
            shallow_boundary.push(commit_oid);
            continue;
        }

        for parent in parents {
            queue.push_back((parent, current_depth + 1));
        }
    }

    Ok(DepthState {
        included_objects,
        shallow_boundary,
    })
}

fn collect_tree_oids(
    repo: &gix::Repository,
    root_tree_oid: gix::ObjectId,
    seen: &mut HashSet<gix::ObjectId>,
    oids: &mut Vec<gix::ObjectId>,
) -> Result<()> {
    let mut stack = vec![root_tree_oid];

    while let Some(tree_oid) = stack.pop() {
        if !seen.insert(tree_oid) {
            continue;
        }

        let tree_obj = repo
            .find_object(tree_oid)
            .map_err(|e| Error::Protocol(e.to_string()))?;
        oids.push(tree_oid);

        for entry_result in gix::objs::TreeRefIter::from_bytes(&tree_obj.data) {
            let entry = entry_result.map_err(|e| Error::Protocol(e.to_string()))?;
            let entry_oid = entry.oid.to_owned();
            let entry_mode = entry.mode;

            if entry_mode.is_tree() {
                stack.push(entry_oid);
            } else if seen.insert(entry_oid) && !entry_mode.is_commit() {
                oids.push(entry_oid);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkt(data: &str) -> Vec<u8> {
        pktline::encode(data.as_bytes())
    }

    #[test]
    fn parse_ls_refs_command() {
        let mut body = Vec::new();
        body.extend_from_slice(&pkt("command=ls-refs\n"));
        body.extend_from_slice(b"0001");
        body.extend_from_slice(&pkt("peel\n"));
        body.extend_from_slice(&pkt("symrefs\n"));
        body.extend_from_slice(&pkt("ref-prefix refs/heads/\n"));
        body.extend_from_slice(b"0000");

        let Command::LsRefs(req) = parse_command_request(&body).unwrap() else {
            panic!("expected ls-refs command");
        };
        assert!(req.peel);
        assert!(req.symrefs);
        assert_eq!(req.ref_prefixes, vec!["refs/heads/"]);
    }

    #[test]
    fn parse_fetch_command() {
        let mut body = Vec::new();
        body.extend_from_slice(&pkt("command=fetch\n"));
        body.extend_from_slice(b"0001");
        body.extend_from_slice(&pkt("ofs-delta\n"));
        body.extend_from_slice(&pkt("want 0000000000000000000000000000000000000001\n"));
        body.extend_from_slice(&pkt("done\n"));
        body.extend_from_slice(b"0000");

        let Command::Fetch(req) = parse_command_request(&body).unwrap() else {
            panic!("expected fetch command");
        };
        assert_eq!(req.upload_request.wants.len(), 1);
        assert!(req.upload_request.done);
        assert!(req.upload_request.capabilities.ofs_delta);
    }

    #[test]
    fn ls_refs_returns_unborn_head() {
        let root = tempfile::TempDir::new().unwrap();
        let repo_path = root.path().join("repo.git");
        std::process::Command::new("git")
            .args(["init", "--bare", repo_path.to_str().unwrap()])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["symbolic-ref", "HEAD", "refs/heads/main"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let out = ls_refs(
            &repo_path,
            &LsRefsRequest {
                unborn: true,
                symrefs: true,
                ..Default::default()
            },
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("unborn HEAD symref-target:refs/heads/main"));
    }
}
