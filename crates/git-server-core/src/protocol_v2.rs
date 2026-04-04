use std::collections::BTreeSet;
use std::collections::HashSet;
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
        _ => Err(Error::Protocol(format!("unsupported protocol v2 command: {command}"))),
    }
}

pub fn ls_refs(repo_path: &Path, request: &LsRefsRequest) -> Result<Vec<u8>> {
    let repo = gix::open(repo_path)?;
    let mut refs = BTreeSet::new();

    if let Ok(mut head) = repo.head()
        && let Some(id) = head.try_peel_to_id().map_err(|e| Error::Protocol(e.to_string()))?
    {
        let mut line = format!("{} HEAD", id.detach());
        if request.symrefs
            && let Some(target) = head.referent_name()
        {
            line.push_str(&format!(" symref-target:{}", target.as_bstr()));
        }
        refs.insert(line);
    }

    if let Ok(platform) = repo.references()
        && let Ok(iter) = platform.all()
    {
        for mut reference in iter.flatten() {
            let name = reference.name().as_bstr().to_string();
            if !request.ref_prefixes.is_empty()
                && !request.ref_prefixes.iter().any(|prefix| name.starts_with(prefix))
            {
                continue;
            }

            let mut line = match reference.peel_to_id() {
                Ok(id) => format!("{} {name}", id.detach()),
                Err(_) => continue,
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

        if payload.starts_with(&[0x01]) || payload.starts_with(&[0x02]) || payload.starts_with(&[0x03]) {
            out.extend_from_slice(frame);
        }
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

pub fn common_haves(repo_path: &Path, request: &FetchRequest) -> Result<Vec<gix::ObjectId>> {
    let repo = gix::open(repo_path)?;
    let want_set: HashSet<gix::ObjectId> = collect_want_closure(&repo, &request.upload_request.wants)?
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
                    return Err(Error::Protocol(format!("unsupported ls-refs argument: {arg}")));
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

    for arg in args {
        if arg == "done" {
            done = true;
        } else if arg == "ofs-delta" {
            capabilities.ofs_delta = true;
        } else if arg == "thin-pack"
            || arg == "no-progress"
            || arg == "include-tag"
            || arg == "wait-for-done"
        {
            continue;
        } else if let Some(oid_hex) = arg.strip_prefix("want ") {
            let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                .map_err(|_| Error::Protocol(format!("invalid OID in want: {oid_hex}")))?;
            wants.push(oid);
        } else if let Some(oid_hex) = arg.strip_prefix("have ") {
            let oid = gix::ObjectId::from_hex(oid_hex.as_bytes())
                .map_err(|_| Error::Protocol(format!("invalid OID in have: {oid_hex}")))?;
            haves.push(oid);
        } else {
            return Err(Error::Protocol(format!("unsupported fetch argument: {arg}")));
        }
    }

    Ok(Command::Fetch(FetchRequest {
        upload_request: UploadPackRequest {
            wants,
            haves,
            done,
            capabilities,
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
}
