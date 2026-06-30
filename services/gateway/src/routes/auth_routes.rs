use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use velora_proto::auth::v1::{
    AuthTokens, LoginRequest, LogoutRequest, RefreshSessionRequest, RegisterRequest,
};

use crate::{error::ApiError, state::AppState};

// --- REST DTOs ---------------------------------------------
// The gateway does not validate the business logic (that is the role of the
// auth-service domain): it simply acts as a transport layer. Duplicate validation here
// would eventually diverge from the actual rule.

#[derive(Deserialize)]
pub struct CredentialsBody {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RefreshBody {
    refresh_token: String,
}

#[derive(Serialize)]
pub struct TokensResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    token_type: &'static str,
}

impl From<AuthTokens> for TokensResponse {
    fn from(t: AuthTokens) -> Self {
        Self {
            access_token: t.access_token,
            refresh_token: t.refresh_token,
            expires_in: t.expires_in,
            token_type: "Bearer",
        }
    }
}

// --- Handlers ----------------------------------------------------
// The same pattern applies everywhere: DTO → proto message → gRPC call
// → proto response → DTO. Errors are propagated via `?` thanks to ApiError's `From<tonic::Status>`.

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<CredentialsBody>,
) -> Result<Json<TokensResponse>, ApiError> {
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

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<CredentialsBody>,
) -> Result<Json<TokensResponse>, ApiError> {
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

pub async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> Result<Json<TokensResponse>, ApiError> {
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

pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<RefreshBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .auth_client
        .clone()
        .logout(LogoutRequest {
            refresh_token: body.refresh_token,
        })
        .await?;

    Ok(Json(serde_json::json!({"logged_out": true})))
}
