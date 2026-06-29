use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// The single error exposed by the REST API.
/// All gRPC-to-HTTP translation happens HERE, and nowhere else.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{1}")]
    Status(StatusCode, String),

    #[error("missing or invalid token")]
    Unauthorized,

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

/// Systematic translation of gRPC status to HTTP.
impl From<tonic::Status> for ApiError {
    fn from(status: tonic::Status) -> Self {
        use tonic::Code;
        let http = match status.code() {
            Code::InvalidArgument => StatusCode::BAD_REQUEST, // 400
            Code::Unauthenticated => StatusCode::UNAUTHORIZED, // 401,
            Code::PermissionDenied => StatusCode::FORBIDDEN,  // 403,
            Code::NotFound => StatusCode::NOT_FOUND,          // 404
            Code::AlreadyExists => StatusCode::CONFLICT,      // 409
            Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS, // 429
            Code::Unavailable => StatusCode::BAD_GATEWAY,     // 502
            _ => StatusCode::INTERNAL_SERVER_ERROR,           // 500
        };
        // Service messages are BUSINESS messages
        // (written to be readable): they are passed along as-is
        Self::Status(http, status.message().to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (code, message) = match self {
            Self::Status(code, msg) => (code, msg),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "missing or invalid token".into()),
            Self::Internal(e) => {
                tracing::error!(error = ?e, "gateway internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
        };

        (code, Json(json!({ "error": message}))).into_response()
    }
}
