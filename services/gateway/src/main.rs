pub mod auth;
pub mod config;
pub mod error;
pub mod routes;
pub mod state;

use auth::PasetoVerifier;
use state::AppState;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    velora_common::telemetry::init("gateway");
    let cfg = config::Config::load()?;

    let paseto = PasetoVerifier::from_hex(&cfg.paseto_public_hex)?;
    let state = AppState::new(cfg.auth_grpc_url, paseto)?;

    let app = routes::build(state)
        // Each HTTP request becomes a trace span (method, URI, status)
        .layer(TraceLayer::new_for_http())
        // Permissive CORS in development only: the frontend runs on a different port.
        // In production: strict origin whitelisting.
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cfg.http_port)).await?;
    tracing::info!(port = cfg.http_port, "gateway listening (REST)");

    axum::serve(listener, app).await?;
    Ok(())
}
