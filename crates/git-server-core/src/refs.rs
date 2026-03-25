use std::path::Path;

use crate::error::Result;
use crate::pktline;

const CAPABILITIES: &str = "side-band-64k";

const ZERO_OID: &str = "0000000000000000000000000000000000000000";

/// Generate the complete pkt-line encoded ref advertisement response
/// for the Git smart HTTP protocol (`git-upload-pack`).
///
/// The output follows the format expected by `git clone` / `git fetch`:
///
/// ```text
/// 001e# service=git-upload-pack\n
/// 0000
/// <first-ref>\0<capabilities>\n
/// <ref-line>\n
/// ...
/// 0000
/// ```
pub fn advertise_refs(repo_path: &Path) -> Result<Vec<u8>> {
    let repo = gix::open(repo_path)?;

    let mut output = Vec::new();

    // Service header + flush
    output.extend_from_slice(&pktline::encode_comment("service=git-upload-pack"));
    output.extend_from_slice(pktline::flush());

    // Collect refs: (oid_hex, refname)
    let mut refs: Vec<(String, String)> = Vec::new();

    // Try to resolve HEAD first
    if let Ok(id) = repo.head_id() {
        refs.push((id.to_string(), "HEAD".to_string()));
    }

    // Iterate all references
    if let Ok(platform) = repo.references()
        && let Ok(iter) = platform.all()
    {
        for mut r in iter.flatten() {
            let name = r.name().as_bstr().to_string();
            if let Ok(id) = r.peel_to_id() {
                refs.push((id.to_string(), name));
            }
        }
    }

    if refs.is_empty() {
        // No refs at all: advertise capabilities with zero OID
        let line = format!("{ZERO_OID} capabilities^{{}}\0{CAPABILITIES}\n");
        output.extend_from_slice(&pktline::encode(line.as_bytes()));
    } else {
        // First ref line includes capabilities after NUL byte
        let (oid, name) = &refs[0];
        let first_line = format!("{oid} {name}\0{CAPABILITIES}\n");
        output.extend_from_slice(&pktline::encode(first_line.as_bytes()));

        // Subsequent ref lines
        for (oid, name) in &refs[1..] {
            let line = format!("{oid} {name}\n");
            output.extend_from_slice(&pktline::encode(line.as_bytes()));
        }
    }

    // Final flush
    output.extend_from_slice(pktline::flush());

    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use tempfile::TempDir;

    use super::*;

    /// Create a bare repo with a single commit on the `main` branch.
    ///
    /// Returns the path to the bare repository.
    fn create_repo_with_commit(root: &Path) -> PathBuf {
        let bare_path = root.join("test.git");
        let clone_path = root.join("workdir");

        // Create a bare repository
        let out = Command::new("git")
            .args(["init", "--bare", bare_path.to_str().unwrap()])
            .output()
            .expect("git init --bare failed");
        assert!(out.status.success(), "git init --bare failed: {:?}", out);

        // Set default branch to main in bare repo
        let out = Command::new("git")
            .args(["symbolic-ref", "HEAD", "refs/heads/main"])
            .current_dir(&bare_path)
            .output()
            .expect("git symbolic-ref failed");
        assert!(out.status.success());

        // Clone the bare repo into a working directory
        let out = Command::new("git")
            .args([
                "clone",
                bare_path.to_str().unwrap(),
                clone_path.to_str().unwrap(),
            ])
            .output()
            .expect("git clone failed");
        assert!(out.status.success(), "git clone failed: {:?}", out);

        // Configure user in the clone
        for (key, val) in [("user.name", "Test User"), ("user.email", "test@test.com")] {
            Command::new("git")
                .args(["config", key, val])
                .current_dir(&clone_path)
                .output()
                .expect("git config failed");
        }

        // Create a commit
        let out = Command::new("git")
            .args(["commit", "--allow-empty", "-m", "initial commit"])
            .current_dir(&clone_path)
            .env("GIT_AUTHOR_NAME", "Test User")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "Test User")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .output()
            .expect("git commit failed");
        assert!(out.status.success(), "git commit failed: {:?}", out);

        // Push to the bare repo
        let out = Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&clone_path)
            .output()
            .expect("git push failed");
        assert!(out.status.success(), "git push failed: {:?}", out);

        bare_path
    }

    #[test]
    fn advertise_refs_starts_with_service_header() {
        let dir = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(dir.path());
        let output = advertise_refs(&repo_path).unwrap();
        let output_str = String::from_utf8_lossy(&output);

        assert!(
            output_str.starts_with("001e# service=git-upload-pack\n0000"),
            "output does not start with service header: {output_str:?}"
        );
    }

    #[test]
    fn advertise_refs_contains_capabilities() {
        let dir = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(dir.path());
        let output = advertise_refs(&repo_path).unwrap();
        let output_str = String::from_utf8_lossy(&output);

        assert!(
            output_str.contains("side-band-64k"),
            "output missing side-band-64k: {output_str:?}"
        );
        assert!(
            !output_str.contains("thin-pack"),
            "output should not advertise thin-pack: {output_str:?}"
        );
        assert!(
            !output_str.contains("ofs-delta"),
            "output should not advertise ofs-delta: {output_str:?}"
        );
        assert!(
            !output_str.contains("shallow"),
            "output should not advertise shallow: {output_str:?}"
        );
    }

    #[test]
    fn advertise_refs_contains_main_branch() {
        let dir = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(dir.path());
        let output = advertise_refs(&repo_path).unwrap();
        let output_str = String::from_utf8_lossy(&output);

        assert!(
            output_str.contains("refs/heads/main"),
            "output missing refs/heads/main: {output_str:?}"
        );
    }

    #[test]
    fn advertise_refs_ends_with_flush() {
        let dir = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(dir.path());
        let output = advertise_refs(&repo_path).unwrap();

        assert!(
            output.ends_with(b"0000"),
            "output does not end with flush packet: {:?}",
            String::from_utf8_lossy(&output)
        );
    }
}
