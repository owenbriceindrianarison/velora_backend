use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use velora_proto::auth::v1::{
    AuthTokens, LoginRequest, LogoutRequest, RefreshSessionRequest, RegisterRequest,
};

use crate::{error::ApiError, state::AppState};

// --- REST DTOs ---------------------------------------------
// Structural validation only (shape of the request): empty fields, email syntax.
// Business rules (password strength, email uniqueness…) stay in the auth-service domain.

#[derive(Deserialize, ToSchema)]
pub struct CredentialsBody {
    /// User email address
    pub email: String,
    /// User password (min 8 chars enforced by auth-service)
    pub password: String,
}

impl CredentialsBody {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.email.trim().is_empty() {
            return Err(ApiError::bad_request("email is required", "EMAIL_REQUIRED"));
        }
        let parts: Vec<&str> = self.email.splitn(2, '@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ApiError::bad_request("email is invalid", "EMAIL_INVALID"));
        }
        if self.password.is_empty() {
            return Err(ApiError::bad_request("password is required", "PASSWORD_REQUIRED"));
        }
        Ok(())
    }
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshBody {
    /// Opaque refresh token previously issued by this API
    pub refresh_token: String,
}

impl RefreshBody {
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.refresh_token.is_empty() {
            return Err(ApiError::bad_request("refresh_token is required", "REFRESH_TOKEN_REQUIRED"));
        }
        Ok(())
    }
}

#[derive(Serialize, ToSchema)]
pub struct TokensResponse {
    access_token: String,
    refresh_token: String,
    /// Token lifetime in seconds
    expires_in: i64,
    token_type: String,
}

impl From<AuthTokens> for TokensResponse {
    fn from(t: AuthTokens) -> Self {
        Self {
            access_token: t.access_token,
            refresh_token: t.refresh_token,
            expires_in: t.expires_in,
            token_type: "Bearer".to_string(),
        }
    }
}

// --- Handlers ----------------------------------------------------
// The same pattern applies everywhere: DTO → proto message → gRPC call
// → proto response → DTO. Errors are propagated via `?` thanks to ApiError's `From<tonic::Status>`.

#[utoipa::path(
    post,
    path = "/v1/auth/register",
    tag = "auth",
    request_body = CredentialsBody,
    responses(
        (status = 200, description = "Account created, tokens returned", body = TokensResponse),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already registered"),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<CredentialsBody>,
) -> Result<Json<TokensResponse>, ApiError> {
    body.validate()?;
    let reply = state
        .auth_client
        .clone() // Low-cost clone: same HTTP/2 channel underneath
        .register(RegisterRequest {
            email: body.email,
            password: body.password,
        })
        .await?
        .into_inner();

    Ok(Json(reply.into()))
}

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    tag = "auth",
    request_body = CredentialsBody,
    responses(
        (status = 200, description = "Login successful, tokens returned", body = TokensResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Wrong credentials"),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<CredentialsBody>,
) -> Result<Json<TokensResponse>, ApiError> {
    body.validate()?;
    let reply = state
        .auth_client
        .clone()
        .login(LoginRequest {
            email: body.email,
            password: body.password,
        })
        .await?
        .into_inner();

    Ok(Json(reply.into()))
}

#[utoipa::path(
    post,
    path = "/v1/auth/refresh",
    tag = "auth",
    request_body = RefreshBody,
    responses(
        (status = 200, description = "New token pair issued", body = TokensResponse),
        (status = 401, description = "Refresh token expired or invalid"),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> Result<Json<TokensResponse>, ApiError> {
    body.validate()?;
    let reply = state
        .auth_client
        .clone()
        .refresh_session(RefreshSessionRequest {
            refresh_token: body.refresh_token,
        })
        .await?
        .into_inner();

    Ok(Json(reply.into()))
}

#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    tag = "auth",
    request_body = RefreshBody,
    responses(
        (status = 200, description = "Session revoked"),
        (status = 401, description = "Refresh token invalid or already revoked"),
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    body.validate()?;
    state
        .auth_client
        .clone()
        .logout(LogoutRequest {
            refresh_token: body.refresh_token,
        })
        .await?;

    Ok(Json(serde_json::json!({"logged_out": true})))
}
