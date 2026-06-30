use axum::{
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use utoipa::OpenApi;

use crate::{auth::CurrentUser, state::AppState};

pub mod auth_routes;

// ---------------------------------------------------------------------------
// OpenAPI spec
// ---------------------------------------------------------------------------

#[derive(OpenApi)]
#[openapi(
    info(title = "Velora Gateway API", version = "0.1.0"),
    paths(
        auth_routes::register,
        auth_routes::login,
        auth_routes::refresh,
        auth_routes::logout,
        me,
    ),
    components(schemas(
        auth_routes::CredentialsBody,
        auth_routes::RefreshBody,
        auth_routes::TokensResponse,
    )),
    tags(
        (name = "auth", description = "Authentication — register, login, refresh, logout"),
        (name = "user", description = "Authenticated user info"),
    ),
    modifiers(&BearerSecurityScheme),
)]
struct ApiDoc;

struct BearerSecurityScheme;

impl utoipa::Modify for BearerSecurityScheme {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/docs", get(rapidoc_ui))
        .route("/api-docs/openapi.json", get(openapi_spec))
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

#[utoipa::path(
    get,
    path = "/v1/me",
    tag = "user",
    responses(
        (status = 200, description = "Current authenticated user"),
        (status = 401, description = "Missing or invalid Bearer token"),
    ),
    security(("bearer_auth" = [])),
)]
async fn me(CurrentUser(user_id): CurrentUser) -> Json<serde_json::Value> {
    Json(json!({ "user_id": user_id }))
}

async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

async fn rapidoc_ui() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Velora API</title>
    <script type="module" src="https://unpkg.com/rapidoc/dist/rapidoc-min.js"></script>
  </head>
  <body>
    <rapi-doc
      spec-url="/api-docs/openapi.json"
      render-style="read"
      theme="dark"
      show-header="false"
    ></rapi-doc>
  </body>
</html>"#,
    )
}

