use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

use crate::{auth::CurrentUser, state::AppState};

mod auth_routes;

pub fn build(state: AppState) -> Router {
    Router::new()
        // Public routes
        .route("/v1/auth/register", post(auth_routes::register))
        .route("/v1/auth/login", post(auth_routes::login))
        .route("/v1/auth/refresh", post(auth_routes::refresh))
        .route("/v1/auth/logout", post(auth_routes::logout))
        // Protected routes
        .route("/v1/me", get(me))
        .route("/health", get(|| async { "ok" }))
        .with_state(state)
}

async fn me(CurrentUser(user_id): CurrentUser) -> Json<serde_json::Value> {
    Json(json!({ "user_id": user_id }))
}
