use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

use tempfile::TempDir;

struct Fixture {
    _dir: TempDir,
    bare_path: PathBuf,
}

static SMALL: LazyLock<Fixture> = LazyLock::new(|| create_fixture(5, 10, 1024, &[], &[]));

static MEDIUM: LazyLock<Fixture> = LazyLock::new(|| create_fixture(200, 100, 5 * 1024, &[], &[]));

static LARGE: LazyLock<Fixture> = LazyLock::new(|| {
    create_fixture(
        2000,
        500,
        10 * 1024,
        &[("feature-a", 500), ("feature-b", 1000)],
        &[
            ("v0.1", 400),
            ("v0.2", 800),
            ("v0.3", 1200),
            ("v0.4", 1600),
            ("v0.5", 2000),
        ],
    )
});

pub fn small_repo() -> &'static Path {
    &SMALL.bare_path
}

pub fn medium_repo() -> &'static Path {
    &MEDIUM.bare_path
}

pub fn large_repo() -> &'static Path {
    &LARGE.bare_path
}

fn create_fixture(
    commit_count: usize,
    file_count: usize,
    file_size: usize,
    branches: &[(&str, usize)],
    tags: &[(&str, usize)],
) -> Fixture {
    let dir = TempDir::new().expect("create temp dir");
    let bare_path = dir.path().join("repo.git");
    let work_dir = dir.path().join("work");

    run_git(&["init", "--bare", bare_path.to_str().unwrap()], None);
    run_git(
        &["symbolic-ref", "HEAD", "refs/heads/main"],
        Some(&bare_path),
    );
    run_git(
        &[
            "clone",
            bare_path.to_str().unwrap(),
            work_dir.to_str().unwrap(),
        ],
        None,
    );
    run_git(&["config", "user.name", "Bench"], Some(&work_dir));
    run_git(&["config", "user.email", "bench@test.com"], Some(&work_dir));

    // Create initial files
    for i in 0..file_count {
        let content = "x".repeat(file_size);
        std::fs::write(work_dir.join(format!("file{i}.txt")), &content).expect("write file");
    }
    run_git(&["add", "."], Some(&work_dir));
    run_git(&["commit", "-m", "initial: add all files"], Some(&work_dir));

    // Subsequent commits: modify files in rotation
    for c in 1..commit_count {
        let file_idx = c % file_count;
        let content = format!("commit {c}\n{}", "y".repeat(file_size));
        std::fs::write(work_dir.join(format!("file{file_idx}.txt")), &content).expect("write file");
        run_git(&["add", "."], Some(&work_dir));
        run_git(&["commit", "-m", &format!("commit {c}")], Some(&work_dir));

        for (name, at) in branches {
            if c == *at {
                run_git(&["branch", name], Some(&work_dir));
            }
        }

        for (name, at) in tags {
            if c == *at {
                run_git(&["tag", name], Some(&work_dir));
            }
        }
    }

    // Push all refs
    run_git(&["push", "origin", "--all"], Some(&work_dir));
    run_git(&["push", "origin", "--tags"], Some(&work_dir));

    Fixture {
        _dir: dir,
        bare_path,
    }
}

fn run_git(args: &[&str], cwd: Option<&Path>) {
    let mut cmd = Command::new("git");
    cmd.args(args)
        .env("GIT_AUTHOR_NAME", "Bench")
        .env("GIT_AUTHOR_EMAIL", "bench@test.com")
        .env("GIT_COMMITTER_NAME", "Bench")
        .env("GIT_COMMITTER_EMAIL", "bench@test.com");
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd.output().expect("git command failed");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
