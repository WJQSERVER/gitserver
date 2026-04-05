use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
};
use serde::Deserialize;
use serde::Serialize;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;

use git_server_core::{backend::GitBackend, discovery::RepoInfo};

use crate::{SharedState, error::AppError};

#[derive(Serialize)]
pub struct HealthzResponse {
    status: &'static str,
}

/// GET / -- list all discovered repositories
pub async fn list_repos(State(store): State<SharedState>) -> Json<Vec<RepoInfo>> {
    Json(store.list().await)
}

/// GET /healthz -- lightweight readiness/liveness probe.
pub async fn healthz() -> Json<HealthzResponse> {
    Json(HealthzResponse { status: "ok" })
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

fn is_protocol_v2(headers: &HeaderMap) -> bool {
    headers
        .get("git-protocol")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(':').any(|item| item.trim() == "version=2"))
        .unwrap_or(false)
}

#[derive(Copy, Clone)]
enum CompressionEncoding {
    Gzip,
    Zstd,
}

fn negotiate_info_refs_encoding(headers: &HeaderMap) -> Option<CompressionEncoding> {
    let accept_encoding = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    if accept_encoding.contains("zstd") || accept_encoding.contains("zst") {
        Some(CompressionEncoding::Zstd)
    } else if accept_encoding.contains("gzip") {
        Some(CompressionEncoding::Gzip)
    } else {
        None
    }
}

fn compress_info_refs(body: Vec<u8>, encoding: CompressionEncoding) -> Result<(Vec<u8>, &'static str), AppError> {
    match encoding {
        CompressionEncoding::Gzip => {
            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, &body)
                .map_err(|e| AppError::Internal(format!("failed to gzip response: {e}")))?;
            let compressed = encoder
                .finish()
                .map_err(|e| AppError::Internal(format!("failed to finish gzip response: {e}")))?;
            Ok((compressed, "gzip"))
        }
        CompressionEncoding::Zstd => {
            let compressed = zstd::stream::encode_all(std::io::Cursor::new(body), 3)
                .map_err(|e| AppError::Internal(format!("failed to zstd response: {e}")))?;
            Ok((compressed, "zstd"))
        }
    }
}

fn require_auth(store: &SharedState, headers: &HeaderMap) -> Result<(), AppError> {
    let auth = store.auth();
    if auth.basic.is_none() && auth.bearer_token.is_none() {
        return Ok(());
    }

    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return Err(AppError::Unauthorized);
    };

    if let Some(token) = value.strip_prefix("Bearer ")
        && auth.bearer_token.as_deref() == Some(token)
    {
        return Ok(());
    }

    if let Some(encoded) = value.strip_prefix("Basic ")
        && let Some(config) = &auth.basic
    {
        let decoded = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .map_err(|_| AppError::Unauthorized)?;
        let decoded = String::from_utf8(decoded).map_err(|_| AppError::Unauthorized)?;
        if decoded == format!("{}:{}", config.username, config.password) {
            return Ok(());
        }
    }

    Err(AppError::Unauthorized)
}

/// GET /{*path} -- dispatches to info_refs when path ends with /info/refs
pub async fn info_refs_dispatch(
    State(store): State<SharedState>,
    Path(path): Path<String>,
    headers: HeaderMap,
    Query(query): Query<InfoRefsQuery>,
) -> Result<Response, AppError> {
    let repo_path = strip_path_suffix(&path, "/info/refs")
        .ok_or_else(|| AppError::NotFound(format!("not found: /{path}")))?;
    info_refs_inner(&store, repo_path, headers, query).await
}

async fn info_refs_inner(
    store: &SharedState,
    repo_path: &str,
    headers: HeaderMap,
    query: InfoRefsQuery,
) -> Result<Response, AppError> {
    if query.service != "git-upload-pack" {
        return Err(AppError::BadRequest(format!(
            "unsupported service: {}",
            query.service
        )));
    }
    require_auth(store, &headers)?;
    let body = if is_protocol_v2(&headers) {
        git_server_core::protocol_v2::advertise_capabilities()
    } else {
        let repo_info = store.resolve(repo_path).await?;
        let backend = GitBackend::new(repo_info.absolute_path.clone());
        backend.advertise_refs()?
    };
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/x-git-upload-pack-advertisement",
        )
        .header(header::CACHE_CONTROL, "no-cache");

    let body = if let Some(encoding) = negotiate_info_refs_encoding(&headers) {
        let (compressed, content_encoding) = compress_info_refs(body, encoding)?;
        builder = builder
            .header(header::CONTENT_ENCODING, content_encoding)
            .header(header::VARY, "Accept-Encoding");
        compressed
    } else {
        body
    };

    Ok(builder.body(Body::from(body)).unwrap())
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
    require_auth(store, &headers)?;
    let repo_info = store.resolve(repo_path).await?;
    let backend = GitBackend::new(repo_info.absolute_path.clone());

    if is_protocol_v2(&headers) {
        return upload_pack_v2(repo_info.absolute_path.as_path(), &backend, &request).await;
    }

    let upload_request = git_server_core::pack::UploadPackRequest::parse(&request)?;
    let reader = backend.upload_pack(&upload_request).await.map_err(|e| {
        tracing::error!("pack generation failed: {e}");
        AppError::Internal("internal server error".into())
    })?;
    let stream = ReaderStream::new(reader);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from_stream(stream))
        .unwrap())
}

async fn upload_pack_v2(
    repo_path: &std::path::Path,
    backend: &GitBackend,
    request: &[u8],
) -> Result<Response, AppError> {
    match git_server_core::protocol_v2::parse_command_request(request)? {
        git_server_core::protocol_v2::Command::LsRefs(req) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(git_server_core::protocol_v2::ls_refs(repo_path, &req)?))
            .unwrap()),
        git_server_core::protocol_v2::Command::Fetch(mut req) => {
            let shallow_update = git_server_core::protocol_v2::apply_shallow_boundaries(repo_path, &mut req)?;
            let is_shallow_negotiation = req.upload_request.shallow.depth.is_some();

            if !req.upload_request.done && !is_shallow_negotiation {
                let common = git_server_core::protocol_v2::common_haves(repo_path, &req)?;
                let mut body = git_server_core::protocol_v2::encode_fetch_acknowledgments(&common);
                body.extend_from_slice(&git_server_core::protocol_v2::encode_shallow_info(&shallow_update));

                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
                    .header(header::CACHE_CONTROL, "no-cache")
                    .body(Body::from(body))
                    .unwrap());
            }

            let mut reader = backend.upload_pack(&req.upload_request).await.map_err(|e| {
                tracing::error!("pack generation failed: {e}");
                AppError::Internal("internal server error".into())
            })?;
            let mut pack = Vec::new();
            reader.read_to_end(&mut pack).await.map_err(|e| {
                tracing::error!("failed reading generated pack: {e}");
                AppError::Internal("internal server error".into())
            })?;

            let mut body = if is_shallow_negotiation && !req.upload_request.haves.is_empty() && !req.upload_request.done {
                git_server_core::protocol_v2::encode_fetch_ready_and_acknowledgments(
                    &git_server_core::protocol_v2::common_haves(repo_path, &req)?,
                )
            } else {
                Vec::new()
            };
            body.extend_from_slice(&git_server_core::protocol_v2::encode_shallow_info(&shallow_update));
            body.extend_from_slice(&git_server_core::protocol_v2::encode_fetch_pack_response(&pack));

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
                .header(header::CACHE_CONTROL, "no-cache")
                .body(Body::from(body))
                .unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
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
        let app = router(crate::SharedState::new(store));

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
    async fn healthz_returns_ok_json() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let app = router(crate::SharedState::new(store));

        let response = app
            .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn info_refs_requires_service_param() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let app = router(crate::SharedState::new(store));

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
        let app = router(crate::SharedState::new(store));

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
        let app = router(crate::SharedState::new(store));

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
        let app = router(crate::SharedState::new(store));

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

    #[tokio::test]
    async fn info_refs_requires_auth_when_configured() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let state = crate::SharedState::with_auth(
            store,
            crate::AuthConfig {
                basic: Some(crate::BasicAuthConfig {
                    username: "alice".into(),
                    password: "secret".into(),
                }),
                bearer_token: None,
            },
        );
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test.git/info/refs?service=git-upload-pack")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().contains_key(header::WWW_AUTHENTICATE));
    }

    #[tokio::test]
    async fn info_refs_accepts_bearer_auth_when_configured() {
        let tmp = TempDir::new().unwrap();
        let store = test_store(&tmp);
        let state = crate::SharedState::with_auth(
            store,
            crate::AuthConfig {
                basic: None,
                bearer_token: Some("token-123".into()),
            },
        );
        let app = router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test.git/info/refs?service=git-upload-pack")
                    .header(header::AUTHORIZATION, "Bearer token-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
