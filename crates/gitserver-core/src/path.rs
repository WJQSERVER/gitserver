use std::path::{Component, Path, PathBuf};

use crate::error::{Error, Result};

/// Normalize a path by resolving `.` and `..` components lexically,
/// without touching the filesystem.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

/// Resolve a relative repo path against a root directory.
/// Returns the canonical absolute path if it is within root.
pub fn resolve_repo_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let candidate = root.join(relative);

    let canonical_root = root
        .canonicalize()
        .map_err(|_| Error::RepoNotFound(relative.to_string()))?;

    // Lexically normalize the candidate to detect traversal before hitting the filesystem.
    let normalized = normalize(&canonical_root.join(relative));
    if !normalized.starts_with(&canonical_root) {
        return Err(Error::PathTraversal(candidate));
    }

    let canonical = candidate
        .canonicalize()
        .map_err(|_| Error::RepoNotFound(relative.to_string()))?;

    if !canonical.starts_with(&canonical_root) {
        return Err(Error::PathTraversal(candidate));
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolve_simple_path() {
        let root = TempDir::new().unwrap();
        let repo_dir = root.path().join("myrepo.git");
        std::fs::create_dir(&repo_dir).unwrap();

        let resolved = resolve_repo_path(root.path(), "myrepo.git").unwrap();
        assert_eq!(resolved, repo_dir.canonicalize().unwrap());
    }

    #[test]
    fn resolve_nested_path() {
        let root = TempDir::new().unwrap();
        let repo_dir = root.path().join("org/project.git");
        std::fs::create_dir_all(&repo_dir).unwrap();

        let resolved = resolve_repo_path(root.path(), "org/project.git").unwrap();
        assert_eq!(resolved, repo_dir.canonicalize().unwrap());
    }

    #[test]
    fn reject_traversal() {
        let root = TempDir::new().unwrap();
        let err = resolve_repo_path(root.path(), "../etc/passwd").unwrap_err();
        assert!(matches!(err, Error::PathTraversal(_)));
    }

    #[test]
    fn reject_traversal_in_middle() {
        let root = TempDir::new().unwrap();
        let repo_dir = root.path().join("legit");
        std::fs::create_dir(&repo_dir).unwrap();

        let err = resolve_repo_path(root.path(), "legit/../../etc/passwd").unwrap_err();
        assert!(matches!(err, Error::PathTraversal(_)));
    }

    #[test]
    fn reject_nonexistent_path() {
        let root = TempDir::new().unwrap();
        let err = resolve_repo_path(root.path(), "nonexistent.git").unwrap_err();
        assert!(matches!(err, Error::RepoNotFound(_)));
    }
}
