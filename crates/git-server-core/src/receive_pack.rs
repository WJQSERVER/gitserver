use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

const RECEIVE_PACK_CONFIG: &[(&str, &str)] = &[
    ("receive.denyDeletes", "true"),
    ("receive.denyNonFastForwards", "true"),
];

pub fn advertise_receive_refs(repo_path: &Path) -> Result<Vec<u8>> {
    run_receive_pack(repo_path, &["--stateless-rpc", "--advertise-refs"], &[])
}

pub fn receive_pack(repo_path: &Path, request: &[u8]) -> Result<Vec<u8>> {
    run_receive_pack(repo_path, &["--stateless-rpc"], request)
}

fn run_receive_pack(repo_path: &Path, args: &[&str], input: &[u8]) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    for (key, value) in RECEIVE_PACK_CONFIG {
        command.args(["-c", &format!("{key}={value}")]);
    }
    command.arg("receive-pack");

    let mut child = command
        .args(args)
        .arg(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if !input.is_empty() {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Error::Protocol("failed to open stdin for git receive-pack".into()))?;
        stdin.write_all(input)?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(Error::Protocol(format!(
            "git receive-pack failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
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
}
