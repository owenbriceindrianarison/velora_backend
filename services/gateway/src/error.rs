use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// The single error exposed by the REST API.
/// All gRPC-to-HTTP translation happens HERE, and nowhere else.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{message}")]
    Status {
        status: StatusCode,
        message: String,
        code: Option<String>,
    },

    #[error("missing or invalid token")]
    Unauthorized,

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl ApiError {
    /// Constructor for gateway-level validation errors (structural checks).
    pub fn bad_request(message: impl Into<String>, code: &'static str) -> Self {
        Self::Status {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
            code: Some(code.to_string()),
        }
    }
}

/// Systematic translation of gRPC status to HTTP.
/// The `x-error-code` metadata set by the upstream service is forwarded as-is.
impl From<tonic::Status> for ApiError {
    fn from(status: tonic::Status) -> Self {
        use tonic::Code;
        let http = match status.code() {
            Code::InvalidArgument => StatusCode::BAD_REQUEST, // 400
            Code::Unauthenticated => StatusCode::UNAUTHORIZED, // 401
            Code::PermissionDenied => StatusCode::FORBIDDEN,  // 403
            Code::NotFound => StatusCode::NOT_FOUND,          // 404
            Code::AlreadyExists => StatusCode::CONFLICT,      // 409
            Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS, // 429
            Code::Unavailable => StatusCode::BAD_GATEWAY,     // 502
            _ => StatusCode::INTERNAL_SERVER_ERROR,           // 500
        };
        let code = status
            .metadata()
            .get("x-error-code")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        Self::Status {
            status: http,
            message: status.message().to_string(),
            code,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (http_status, message, code) = match self {
            Self::Status {
                status,
                message,
                code,
            } => (status, message, code),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "missing or invalid token".into(),
                Some("UNAUTHORIZED".to_string()),
            ),
            Self::Internal(e) => {
                tracing::error!(error = ?e, "gateway internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".into(),
                    None,
                )
            }
        };

        let mut body = json!({ "error": message });
        if let Some(c) = code {
            body["code"] = json!(c);
        }
        (http_status, Json(body)).into_response()
    }
}
