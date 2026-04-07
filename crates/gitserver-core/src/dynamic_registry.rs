// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2026 WJQSERVER

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::discovery::{RepoInfo, RepoStore};
use crate::error::{Error, Result};

pub trait RepoResolver: Send + Sync {
    fn resolve(&self, relative: &str) -> Result<RepoInfo>;
    fn list(&self) -> Result<Vec<RepoInfo>>;
}

pub trait MutableRepoRegistry: RepoResolver {
    fn register(&self, repo: RepoInfo) -> Result<()>;
    fn unregister(&self, relative: &str) -> Result<()>;
}

#[derive(Clone, Default)]
pub struct DynamicRepoRegistry {
    repos: Arc<RwLock<Vec<RepoInfo>>>,
}

impl DynamicRepoRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_repos(repos: Vec<RepoInfo>) -> Result<Self> {
        let registry = Self::new();
        for repo in repos {
            registry.register(repo)?;
        }
        Ok(registry)
    }
}

impl RepoResolver for RepoStore {
    fn resolve(&self, relative: &str) -> Result<RepoInfo> {
        RepoStore::resolve(self, relative).cloned()
    }

    fn list(&self) -> Result<Vec<RepoInfo>> {
        Ok(RepoStore::list(self).to_vec())
    }
}

impl RepoResolver for DynamicRepoRegistry {
    fn resolve(&self, relative: &str) -> Result<RepoInfo> {
        let normalized = normalize_relative_repo_path(relative)?;
        self.repos
            .read()
            .expect("dynamic repo registry poisoned")
            .iter()
            .find(|repo| repo.relative_path == normalized)
            .cloned()
            .ok_or_else(|| Error::RepoNotFound(relative.to_string()))
    }

    fn list(&self) -> Result<Vec<RepoInfo>> {
        Ok(self
            .repos
            .read()
            .expect("dynamic repo registry poisoned")
            .clone())
    }
}

impl MutableRepoRegistry for DynamicRepoRegistry {
    fn register(&self, repo: RepoInfo) -> Result<()> {
        let relative_path = normalize_relative_repo_path(&repo.relative_path)?;
        let repo_path = repo.absolute_path.canonicalize()?;
        let opened = gix::open(&repo_path)?;
        if !opened.is_bare() {
            return Err(Error::Protocol(format!(
                "registered path is not a bare repository: {}",
                repo_path.display()
            )));
        }

        let mut repos = self.repos.write().expect("dynamic repo registry poisoned");
        if repos
            .iter()
            .any(|existing| existing.relative_path == relative_path)
        {
            return Err(Error::Protocol(format!(
                "repository already registered: {}",
                relative_path
            )));
        }

        let mut repo = repo;
        repo.relative_path = relative_path;
        repo.absolute_path = repo_path;
        repos.push(repo);
        repos.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        Ok(())
    }

    fn unregister(&self, relative: &str) -> Result<()> {
        let normalized = normalize_relative_repo_path(relative)?;
        let mut repos = self.repos.write().expect("dynamic repo registry poisoned");
        let original_len = repos.len();
        repos.retain(|repo| repo.relative_path != normalized);
        if repos.len() == original_len {
            return Err(Error::RepoNotFound(relative.to_string()));
        }
        Ok(())
    }
}

fn normalize_relative_repo_path(relative: &str) -> Result<String> {
    let path = Path::new(relative);
    if path.is_absolute() {
        return Err(Error::PathTraversal(relative.to_string().into()));
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => normalized.push(part),
            std::path::Component::CurDir => {}
            _ => return Err(Error::PathTraversal(relative.to_string().into())),
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(Error::RepoNotFound(relative.to_string()));
    }

    Ok(normalized.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use tempfile::TempDir;

    use super::*;

    fn create_bare_repo(path: &Path) {
        Command::new("git")
            .args(["init", "--bare", path.to_str().unwrap()])
            .output()
            .expect("git init --bare failed");
    }

    #[test]
    fn dynamic_registry_registers_and_unregisters() {
        let dir = TempDir::new().unwrap();
        let repo_path = dir.path().join("alpha.git");
        create_bare_repo(&repo_path);

        let registry = DynamicRepoRegistry::new();
        registry
            .register(RepoInfo {
                name: "alpha.git".into(),
                relative_path: "alpha.git".into(),
                absolute_path: repo_path.clone(),
                description: None,
            })
            .unwrap();

        assert_eq!(registry.list().unwrap().len(), 1);
        assert_eq!(
            registry.resolve("alpha.git").unwrap().absolute_path,
            repo_path.canonicalize().unwrap()
        );

        registry.unregister("alpha.git").unwrap();
        assert!(matches!(
            registry.resolve("alpha.git"),
            Err(Error::RepoNotFound(_))
        ));
    }

    #[test]
    fn dynamic_registry_resolve_and_unregister_normalize_paths() {
        let dir = TempDir::new().unwrap();
        let repo_path = dir.path().join("alpha.git");
        create_bare_repo(&repo_path);

        let registry = DynamicRepoRegistry::new();
        registry
            .register(RepoInfo {
                name: "alpha.git".into(),
                relative_path: "alpha.git".into(),
                absolute_path: repo_path,
                description: None,
            })
            .unwrap();

        assert!(registry.resolve("./alpha.git").is_ok());
        registry.unregister("./alpha.git").unwrap();
        assert!(matches!(
            registry.resolve("alpha.git"),
            Err(Error::RepoNotFound(_))
        ));
    }

    #[test]
    fn dynamic_registry_rejects_duplicate_registration() {
        let dir = TempDir::new().unwrap();
        let repo_path = dir.path().join("alpha.git");
        create_bare_repo(&repo_path);

        let registry = DynamicRepoRegistry::new();
        let repo = RepoInfo {
            name: "alpha.git".into(),
            relative_path: "alpha.git".into(),
            absolute_path: repo_path,
            description: None,
        };

        registry.register(repo.clone()).unwrap();
        let err = registry.register(repo).unwrap_err();
        assert!(matches!(err, Error::Protocol(_)));
    }

    #[test]
    fn dynamic_registry_rejects_parent_relative_paths() {
        let dir = TempDir::new().unwrap();
        let repo_path = dir.path().join("alpha.git");
        create_bare_repo(&repo_path);

        let registry = DynamicRepoRegistry::new();
        let err = registry
            .register(RepoInfo {
                name: "alpha.git".into(),
                relative_path: "./team/../alpha.git".into(),
                absolute_path: repo_path,
                description: None,
            })
            .unwrap_err();

        assert!(matches!(err, Error::PathTraversal(_)));
    }

    #[test]
    fn dynamic_registry_rejects_absolute_relative_path() {
        let dir = TempDir::new().unwrap();
        let repo_path = dir.path().join("alpha.git");
        create_bare_repo(&repo_path);

        let registry = DynamicRepoRegistry::new();
        let err = registry
            .register(RepoInfo {
                name: "alpha.git".into(),
                relative_path: "/tmp/alpha.git".into(),
                absolute_path: repo_path,
                description: None,
            })
            .unwrap_err();

        assert!(matches!(err, Error::PathTraversal(_)));
    }
}
