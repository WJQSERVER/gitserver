use std::path::PathBuf;

use tokio::io::AsyncRead;

use crate::error::Result;
use crate::pack::UploadPackRequest;

pub struct GitBackend {
    repo_path: PathBuf,
}

impl GitBackend {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    pub fn advertise_refs(&self) -> Result<Vec<u8>> {
        crate::refs::advertise_refs(&self.repo_path)
    }

    pub async fn upload_pack(&self, request: &UploadPackRequest) -> Result<impl AsyncRead + use<>> {
        crate::pack::generate_pack(&self.repo_path, request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_repo_with_commit(root: &std::path::Path) -> PathBuf {
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
    fn backend_advertise_refs() {
        let root = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(root.path());
        let backend = GitBackend::new(repo_path);
        let output = backend.advertise_refs().unwrap();
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("refs/heads/main"));
    }

    #[tokio::test]
    async fn backend_upload_pack() {
        let root = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(root.path());
        let repo = gix::open(&repo_path).unwrap();
        let head = repo.head_id().unwrap();

        let backend = GitBackend::new(repo_path);
        let request = UploadPackRequest {
            wants: vec![head.detach()],
            haves: vec![],
            done: true,
        };
        let reader = backend.upload_pack(&request).await.unwrap();
        let mut buf = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut tokio::io::BufReader::new(reader), &mut buf)
            .await
            .unwrap();
        assert!(buf.windows(4).any(|w| w == b"PACK"));
    }
}
