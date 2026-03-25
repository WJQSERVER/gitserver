use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use git_server_core::error::Error as CoreError;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
        };

        (
            status,
            Json(ErrorBody {
                error: error_code,
                message,
            }),
        )
            .into_response()
    }
}

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> Self {
        match &err {
            CoreError::RepoNotFound(_) => Self::NotFound(err.to_string()),
            CoreError::PathTraversal(_) | CoreError::Protocol(_) => {
                Self::BadRequest(err.to_string())
            }
            _ => Self::Internal(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;

    async fn read_body_json(body: Body) -> serde_json::Value {
        let bytes = body.collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn not_found_returns_404_json() {
        let err = AppError::NotFound("repo not found".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = read_body_json(response.into_body()).await;
        assert_eq!(json["error"], "not_found");
        assert_eq!(json["message"], "repo not found");
    }

    #[tokio::test]
    async fn bad_request_returns_400_json() {
        let err = AppError::BadRequest("invalid service".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = read_body_json(response.into_body()).await;
        assert_eq!(json["error"], "bad_request");
        assert_eq!(json["message"], "invalid service");
    }

    #[tokio::test]
    async fn internal_error_returns_500_json() {
        let err = AppError::Internal("something went wrong".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let json = read_body_json(response.into_body()).await;
        assert_eq!(json["error"], "internal_error");
        assert_eq!(json["message"], "something went wrong");
    }
}
