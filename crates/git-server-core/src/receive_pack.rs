use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::atomic::AtomicBool;

use gix::objs::bstr::BString;
use gix::prelude::ObjectIdExt;
use gix::progress::Discard;
use gix::refs::transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog};
use gix::refs::Target;

use crate::error::{Error, Result};
use crate::pktline;

const ZERO_ID: &str = "0000000000000000000000000000000000000000";
const CAPABILITIES: &str =
    "report-status report-status-v2 delete-refs side-band-64k quiet atomic ofs-delta object-format=sha1 agent=git-server/0.1";

pub fn advertise_receive_refs(repo_path: &Path) -> Result<Vec<u8>> {
    let repo = gix::open(repo_path)?;
    let mut out = Vec::new();
    let head_name = repo.head_name().ok().flatten();
    let mut refs: Vec<(String, gix::ObjectId)> = repo
        .references()
        .map_err(|e| Error::Protocol(format!("failed to open refs: {e}")))?
        .all()
        .map_err(|e| Error::Protocol(format!("failed to iterate refs: {e}")))?
        .flatten()
        .filter_map(|mut reference| {
            reference
                .peel_to_id()
                .ok()
                .map(|id| (reference.name().as_bstr().to_string(), id.detach()))
        })
        .collect();
    refs.sort_by(|a, b| a.0.cmp(&b.0));

    if refs.is_empty() {
        out.extend_from_slice(&pktline::encode(
            format!("{ZERO_ID} capabilities^{{}}\0{CAPABILITIES}\n").as_bytes(),
        ));
    } else {
        let (first_name, first_id) = &refs[0];
        let mut first = format!("{} {}\0{CAPABILITIES}", first_id, first_name);
        if head_name
            .as_ref()
            .is_some_and(|head| head.as_bstr() == first_name.as_str())
        {
            first.push_str(&format!(" symref=HEAD:{first_name}"));
        }
        first.push('\n');
        out.extend_from_slice(&pktline::encode(first.as_bytes()));

        for (name, id) in refs.into_iter().skip(1) {
            out.extend_from_slice(&pktline::encode(format!("{id} {name}\n").as_bytes()));
        }
    }

    out.extend_from_slice(pktline::flush());
    Ok(out)
}

pub fn receive_pack(repo_path: &Path, request: &[u8]) -> Result<Vec<u8>> {
    let repo = gix::open(repo_path)?;
    let parsed = parse_request(request)?;
    let status = apply_commands(&repo, repo_path, &parsed)?;
    Ok(encode_report_status(&parsed.capabilities, &status))
}

#[derive(Default)]
struct ReceivePackCapabilities {
    report_status: bool,
    report_status_v2: bool,
}

struct ReceivePackRequest {
    commands: Vec<UpdateCommand>,
    pack: Vec<u8>,
    capabilities: ReceivePackCapabilities,
}

struct UpdateCommand {
    old_id: String,
    new_id: String,
    refname: String,
}

enum CommandStatus {
    Ok(String),
    Ng(String, String),
}

fn parse_request(request: &[u8]) -> Result<ReceivePackRequest> {
    let mut pos = 0;
    let mut commands = Vec::new();
    let mut capabilities = ReceivePackCapabilities::default();

    while pos + 4 <= request.len() {
        let len_str = std::str::from_utf8(&request[pos..pos + 4])
            .map_err(|_| Error::Protocol("invalid pkt-line length prefix".into()))?;
        pos += 4;

        if len_str == "0000" {
            break;
        }

        let len = usize::from_str_radix(len_str, 16)
            .map_err(|_| Error::Protocol("invalid pkt-line length".into()))?;
        if len < 4 || pos + (len - 4) > request.len() {
            return Err(Error::Protocol("invalid pkt-line frame length".into()));
        }

        let payload = &request[pos..pos + (len - 4)];
        pos += len - 4;

        let (command_bytes, capability_bytes) =
            if let Some(nul) = payload.iter().position(|b| *b == 0) {
                (&payload[..nul], Some(&payload[nul + 1..]))
            } else {
                (payload, None)
            };

        if let Some(capability_bytes) = capability_bytes {
            let capabilities_line = std::str::from_utf8(capability_bytes)
                .map_err(|_| Error::Protocol("invalid UTF-8 in receive-pack capabilities".into()))?
                .trim_end_matches('\n');
            for capability in capabilities_line.split_ascii_whitespace() {
                match capability {
                    "report-status" => capabilities.report_status = true,
                    "report-status-v2" => {
                        capabilities.report_status = true;
                        capabilities.report_status_v2 = true;
                    }
                    _ => {}
                }
            }
        }

        let line = std::str::from_utf8(command_bytes)
            .map_err(|_| Error::Protocol("invalid UTF-8 in update command".into()))?
            .trim_end_matches('\n');
        let mut parts = line.split_ascii_whitespace();
        let Some(old_id) = parts.next() else { continue };
        let Some(new_id) = parts.next() else { continue };
        let Some(refname) = parts.next() else {
            continue;
        };

        commands.push(UpdateCommand {
            old_id: old_id.to_owned(),
            new_id: new_id.to_owned(),
            refname: refname.to_owned(),
        });
    }

    Ok(ReceivePackRequest {
        commands,
        pack: request[pos..].to_vec(),
        capabilities,
    })
}

fn apply_commands(
    repo: &gix::Repository,
    repo_path: &Path,
    request: &ReceivePackRequest,
) -> Result<Vec<CommandStatus>> {
    if !request.pack.is_empty() {
        write_pack(repo_path, &request.pack)?;
    }

    let mut statuses = Vec::with_capacity(request.commands.len());
    for command in &request.commands {
        match validate_and_update_ref(repo, command) {
            Ok(()) => statuses.push(CommandStatus::Ok(command.refname.clone())),
            Err(err) => statuses.push(CommandStatus::Ng(command.refname.clone(), err.to_string())),
        }
    }
    Ok(statuses)
}

fn write_pack(repo_path: &Path, pack: &[u8]) -> Result<()> {
    if pack.is_empty() {
        return Ok(());
    }

    let mut reader = BufReader::new(Cursor::new(pack));
    let mut progress = Discard;
    let interrupt = AtomicBool::new(false);
    let outcome = gix_pack::Bundle::write_to_directory(
        &mut reader,
        Some(repo_path.join("objects/pack").as_path()),
        &mut progress,
        &interrupt,
        None::<&gix::Repository>,
        Default::default(),
    )
    .map_err(|e| Error::Protocol(format!("failed to write incoming pack: {e}")))?;

    if let Some(keep) = outcome.keep_path {
        let _ = std::fs::remove_file(keep);
    }
    Ok(())
}

fn validate_and_update_ref(repo: &gix::Repository, command: &UpdateCommand) -> Result<()> {
    if command.new_id == ZERO_ID {
        return Err(Error::Protocol(format!(
            "deletion prohibited for {}",
            command.refname
        )));
    }

    let new_id = gix::ObjectId::from_hex(command.new_id.as_bytes())
        .map_err(|_| Error::Protocol(format!("invalid new object id: {}", command.new_id)))?;
    let new_header = repo
        .find_header(new_id)
        .map_err(|e| Error::Protocol(format!("missing new object {}: {e}", command.new_id)))?;
    if new_header.kind() != gix::objs::Kind::Commit {
        return Err(Error::Protocol(format!(
            "updates to {} must point to a commit",
            command.refname
        )));
    }

    let name: gix::refs::FullName = command
        .refname
        .as_str()
        .try_into()
        .map_err(|e| Error::Protocol(format!("invalid ref name {}: {e}", command.refname)))?;

    let (expected, log_message) = if command.old_id == ZERO_ID {
        (PreviousValue::MustNotExist, BString::from("push create"))
    } else {
        let old_id = gix::ObjectId::from_hex(command.old_id.as_bytes())
            .map_err(|_| Error::Protocol(format!("invalid old object id: {}", command.old_id)))?;
        ensure_fast_forward(repo, old_id, new_id, &command.refname)?;
        (
            PreviousValue::MustExistAndMatch(Target::Object(old_id)),
            BString::from("push"),
        )
    };

    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: log_message,
            },
            expected,
            new: Target::Object(new_id),
        },
        name,
        deref: false,
    })
    .map_err(|e| Error::Protocol(format!("failed to update {}: {e}", command.refname)))?;

    Ok(())
}

fn ensure_fast_forward(
    repo: &gix::Repository,
    old_id: gix::ObjectId,
    new_id: gix::ObjectId,
    refname: &str,
) -> Result<()> {
    if old_id == new_id {
        return Ok(());
    }

    let old_commit_time = repo
        .find_object(old_id)
        .map_err(|e| Error::Protocol(format!("failed to inspect current tip for {refname}: {e}")))?
        .try_into_commit()
        .map_err(|_| Error::Protocol(format!("current tip of {refname} is not a commit")))?
        .committer()
        .map_err(|e| Error::Protocol(format!("failed to read commit metadata for {refname}: {e}")))?
        .seconds();

    let mut ancestors = new_id
        .attach(repo)
        .ancestors()
        .sorting(gix::revision::walk::Sorting::ByCommitTimeCutoff {
            order: Default::default(),
            seconds: old_commit_time,
        })
        .all()
        .map_err(|e| Error::Protocol(format!("failed to walk commits for {refname}: {e}")))?;

    if ancestors.any(|id: std::result::Result<_, _>| id.is_ok_and(|commit| commit.id == old_id)) {
        Ok(())
    } else {
        Err(Error::Protocol(format!(
            "non-fast-forward update to {refname} is not allowed"
        )))
    }
}

fn encode_report_status(
    capabilities: &ReceivePackCapabilities,
    statuses: &[CommandStatus],
) -> Vec<u8> {
    if !capabilities.report_status {
        return pktline::flush().to_vec();
    }

    let mut status_lines = Vec::new();
    let unpack_ok = statuses
        .iter()
        .all(|status| matches!(status, CommandStatus::Ok(_)));
    if unpack_ok {
        status_lines.extend_from_slice(&pktline::encode(b"unpack ok\n"));
    } else {
        status_lines.extend_from_slice(&pktline::encode(b"unpack ok\n"));
    }

    for status in statuses {
        match status {
            CommandStatus::Ok(refname) => {
                status_lines
                    .extend_from_slice(&pktline::encode(format!("ok {refname}\n").as_bytes()));
            }
            CommandStatus::Ng(refname, message) => {
                status_lines.extend_from_slice(&pktline::encode(
                    format!("ng {refname} {message}\n").as_bytes(),
                ));
            }
        }
    }
    status_lines.extend_from_slice(pktline::flush());

    if capabilities.report_status_v2 {
        let mut sideband = Vec::new();
        const MAX_BAND_PAYLOAD: usize = 65519;
        for chunk in status_lines.chunks(MAX_BAND_PAYLOAD) {
            let len = 4 + 1 + chunk.len();
            sideband.extend_from_slice(format!("{len:04x}").as_bytes());
            sideband.push(0x01);
            sideband.extend_from_slice(chunk);
        }
        sideband.extend_from_slice(pktline::flush());
        sideband
    } else {
        status_lines
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use tempfile::TempDir;

    use super::*;

    fn create_repo_with_commit(root: &std::path::Path) -> std::path::PathBuf {
        let repo_path = root.join("test.git");
        let work_dir = root.join("work");
        std::fs::create_dir(&work_dir).unwrap();
        Command::new("git")
            .args(["init", "--bare", repo_path.to_str().unwrap()])
            .output()
            .unwrap();
        Command::new("git")
            .args(["symbolic-ref", "HEAD", "refs/heads/main"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        Command::new("git")
            .args([
                "clone",
                repo_path.to_str().unwrap(),
                work_dir.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&work_dir)
            .args(["commit", "--allow-empty", "-m", "init"])
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "t@t.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "t@t.com")
            .output()
            .unwrap();
        Command::new("git")
            .current_dir(&work_dir)
            .args(["push", "origin", "main"])
            .output()
            .unwrap();
        repo_path
    }

    #[test]
    fn advertise_receive_pack_refs() {
        let root = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(root.path());
        let output = advertise_receive_refs(&repo_path).unwrap();
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("refs/heads/main"));
        assert!(output_str.contains("report-status"));
    }

    #[test]
    fn parse_receive_pack_request_with_capabilities() {
        let payload = b"0000000000000000000000000000000000000000 1111111111111111111111111111111111111111 refs/heads/main\0 report-status-v2 side-band-64k\n";
        let mut body = format!("{:04x}", payload.len() + 4).into_bytes();
        body.extend_from_slice(payload);
        body.extend_from_slice(b"0000PACK");

        let parsed = parse_request(&body).unwrap();
        assert_eq!(parsed.commands.len(), 1);
        assert!(parsed.capabilities.report_status);
        assert!(parsed.capabilities.report_status_v2);
        assert_eq!(parsed.pack, b"PACK");
    }
}
