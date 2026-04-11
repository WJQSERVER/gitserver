use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use tokio::io::AsyncRead;
use tokio::time::{Duration, Sleep, sleep};
use tokio_util::io::SyncIoBridge;

use crate::error::Result;
use crate::pack::UploadPackRequest;

pub const RECEIVE_PACK_TIMEOUT: Duration = Duration::from_secs(300);
const RECEIVE_PACK_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Default)]
pub struct UploadPackLimits {
    pub max_pack_bytes: Option<u64>,
}

struct TimedAsyncRead<R> {
    inner: R,
    timeout: Duration,
    sleep: Option<Pin<Box<Sleep>>>,
    interrupt: Arc<AtomicBool>,
}

impl<R> TimedAsyncRead<R> {
    fn new(inner: R, timeout: Duration, interrupt: Arc<AtomicBool>) -> Self {
        Self {
            inner,
            timeout,
            sleep: None,
            interrupt,
        }
    }
}

impl<R> AsyncRead for TimedAsyncRead<R>
where
    R: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.sleep.is_none() {
            self.sleep = Some(Box::pin(sleep(self.timeout)));
        }

        let before = buf.filled().len();
        match Pin::new(&mut self.inner).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                if buf.filled().len() > before {
                    self.sleep = None;
                }
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => {
                if self
                    .sleep
                    .as_mut()
                    .expect("timeout sleep must exist")
                    .as_mut()
                    .poll(cx)
                    .is_ready()
                {
                    self.interrupt.store(true, Ordering::Relaxed);
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "receive-pack read timed out",
                    )))
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

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

    pub fn advertise_receive_refs(&self) -> Result<Vec<u8>> {
        crate::receive_pack::advertise_receive_refs(&self.repo_path)
    }

    pub async fn upload_pack(&self, request: &UploadPackRequest) -> Result<impl AsyncRead + use<>> {
        self.upload_pack_with_limits(request, UploadPackLimits::default())
            .await
    }

    pub async fn upload_pack_with_limits(
        &self,
        request: &UploadPackRequest,
        limits: UploadPackLimits,
    ) -> Result<impl AsyncRead + Send + Unpin + use<>> {
        self.check_pack_size_limit(request, limits.max_pack_bytes)?;
        crate::pack::generate_pack(&self.repo_path, request)
    }

    fn check_pack_size_limit(
        &self,
        request: &UploadPackRequest,
        max_pack_bytes: Option<u64>,
    ) -> Result<()> {
        let Some(limit) = max_pack_bytes else {
            return Ok(());
        };

        let size = crate::pack::estimate_pack_size(&self.repo_path, request)?;
        if size > limit {
            return Err(crate::error::Error::PackTooLarge {
                limit,
                actual: size,
            });
        }

        Ok(())
    }

    pub async fn receive_pack<R>(&self, request: R) -> Result<Vec<u8>>
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        self.receive_pack_with_timeout(request, RECEIVE_PACK_TIMEOUT)
            .await
    }

    pub async fn receive_pack_with_timeout<R>(
        &self,
        request: R,
        timeout_duration: Duration,
    ) -> Result<Vec<u8>>
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        let repo_path = self.repo_path.clone();
        let interrupt = Arc::new(AtomicBool::new(false));
        let watchdog_interrupt = interrupt.clone();
        let watchdog = tokio::spawn(async move {
            sleep(timeout_duration).await;
            watchdog_interrupt.store(true, Ordering::Relaxed);
        });

        let join = tokio::task::spawn_blocking(move || {
            let request =
                TimedAsyncRead::new(request, RECEIVE_PACK_IDLE_TIMEOUT, interrupt.clone());
            let mut request = SyncIoBridge::new(request);
            crate::receive_pack::receive_pack_with_interrupt(
                &repo_path,
                &mut request,
                interrupt.as_ref(),
            )
        })
        .await
        .map_err(|e| crate::error::Error::Protocol(format!("receive-pack task panicked: {e}")));

        watchdog.abort();
        join?
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
            capabilities: Default::default(),
            shallow: Default::default(),
            object_ids: None,
        };
        let reader = backend.upload_pack(&request).await.unwrap();
        let mut buf = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut tokio::io::BufReader::new(reader), &mut buf)
            .await
            .unwrap();
        assert!(buf.windows(4).any(|w| w == b"PACK"));
    }

    #[tokio::test]
    async fn backend_receive_pack_times_out_on_stalled_reader() {
        let root = TempDir::new().unwrap();
        let repo_path = create_repo_with_commit(root.path());
        let backend = GitBackend::new(repo_path);
        let (reader, _writer) = tokio::io::duplex(1);

        let err = backend
            .receive_pack_with_timeout(reader, Duration::from_millis(50))
            .await
            .unwrap_err();

        match err {
            crate::error::Error::Io(inner) => {
                assert_eq!(inner.kind(), std::io::ErrorKind::TimedOut);
            }
            other => panic!("expected timeout io error, got {other}"),
        }
    }
}
