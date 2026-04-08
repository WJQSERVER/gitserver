use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("repository not found: {0}")]
    RepoNotFound(String),

    #[error("path traversal rejected: {0}")]
    PathTraversal(PathBuf),

    #[error("invalid repository at {0}: {1}")]
    InvalidRepo(PathBuf, String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("git operation failed: {0}")]
    Git(Box<gix::open::Error>),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<gix::open::Error> for Error {
    fn from(e: gix::open::Error) -> Self {
        Error::Git(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_repo_not_found() {
        let err = Error::RepoNotFound("foo/bar.git".into());
        assert_eq!(err.to_string(), "repository not found: foo/bar.git");
    }

    #[test]
    fn error_display_path_traversal() {
        let err = Error::PathTraversal(PathBuf::from("../etc/passwd"));
        assert_eq!(err.to_string(), "path traversal rejected: ../etc/passwd");
    }
}
