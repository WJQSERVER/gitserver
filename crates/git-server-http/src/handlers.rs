use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
};
use serde::Deserialize;
use tokio_util::io::ReaderStream;

use git_server_core::{backend::GitBackend, discovery::RepoInfo};

use crate::{SharedState, error::AppError};

/// GET / -- list all discovered repositories
pub async fn list_repos(State(store): State<SharedState>) -> Json<Vec<RepoInfo>> {
    Json(store.list().to_vec())
}

#[derive(Deserialize)]
pub struct InfoRefsQuery {
    service: String,
}

/// Strip a known suffix from a path string, returning the repo path.
///
/// Returns `None` if the path does not end with `suffix`.
fn strip_path_suffix<'a>(path: &'a str, suffix: &str) -> Option<&'a str> {
    path.strip_suffix(suffix).map(|s| s.trim_end_matches('/'))
}

/// GET /{*path} -- dispatches to info_refs when path ends with /info/refs
pub async fn info_refs_dispatch(
    State(store): State<SharedState>,
    Path(path): Path<String>,
    Query(query): Query<InfoRefsQuery>,
) -> Result<Response, AppError> {
    let repo_path = strip_path_suffix(&path, "/info/refs")
        .ok_or_else(|| AppError::NotFound(format!("not found: /{path}")))?;
    info_refs_inner(&store, repo_path, query).await
}

async fn info_refs_inner(
    store: &SharedState,
    repo_path: &str,
    query: InfoRefsQuery,
) -> Result<Response, AppError> {
    if query.service != "git-upload-pack" {
        return Err(AppError::BadRequest(format!(
            "unsupported service: {}",
            query.service
        )));
    }
    let repo_info = store.resolve(repo_path)?;
    let backend = GitBackend::new(repo_info.absolute_path.clone());
    let body = backend.advertise_refs()?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/x-git-upload-pack-advertisement",
        )
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap())
}

/// POST /{*path} -- dispatches to upload_pack when path ends with /git-upload-pack
pub async fn upload_pack_dispatch(
    State(store): State<SharedState>,
    Path(path): Path<String>,
    headers: HeaderMap,
    request: axum::body::Bytes,
) -> Result<Response, AppError> {
    let repo_path = strip_path_suffix(&path, "/git-upload-pack")
        .ok_or_else(|| AppError::NotFound(format!("not found: /{path}")))?;
    upload_pack_inner(&store, repo_path, headers, request).await
}

async fn upload_pack_inner(
    store: &SharedState,
    repo_path: &str,
    headers: HeaderMap,
    request: axum::body::Bytes,
) -> Result<Response, AppError> {
    // Validate Content-Type
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if content_type != "application/x-git-upload-pack-request" {
        return Err(AppError::BadRequest(format!(
            "invalid content type: expected application/x-git-upload-pack-request, got {content_type}"
        )));
    }
    let repo_info = store.resolve(repo_path)?;
    let backend = GitBackend::new(repo_info.absolute_path.clone());
    let upload_request = git_server_core::pack::UploadPackRequest::parse(&request)?;
    let reader = backend
        .upload_pack(&upload_request)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let stream = ReaderStream::new(reader);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from_stream(stream))
        .unwrap())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tempfile::TempDir;
    use tower::ServiceExt;

    use git_server_core::discovery::RepoStore;

    use crate::router;

    fn create_bare_repo(path: &Path) {
        let out = Command::new("git")
            .args(["init", "--bare", path.to_str().unwrap()])
            .output()
            .expect("git init --bare failed");
        assert!(out.status.success());
    }

    fn test_store(tmp: &TempDir) -> RepoStore {
        create_bare_repo(&tmp.path().join("test.git"));
        RepoStore::discover(tmp.path().to_path_buf(), 0).unwrap()
    }

    #[tokio::test]
    async fn list_repos_returns_json() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let app = router(store);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let arr = json.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "test.git");
    }

    #[tokio::test]
    async fn info_refs_requires_service_param() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let app = router(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test.git/info/refs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Missing ?service query param -> 400 (query deserialization failure)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn info_refs_rejects_receive_pack() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let app = router(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test.git/info/refs?service=git-receive-pack")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "bad_request");
    }

    #[tokio::test]
    async fn nonexistent_repo_returns_404() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let app = router(store);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nope.git/info/refs?service=git-upload-pack")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "not_found");
    }

    #[tokio::test]
    async fn upload_pack_rejects_wrong_content_type() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let app = router(store);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/test.git/git-upload-pack")
                    .header("content-type", "application/octet-stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "bad_request");
    }
}
