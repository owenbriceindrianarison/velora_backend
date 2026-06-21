use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initializes the service's structured logs.
///
/// Phase 1: human-readable console output, filtered by RUST_LOG.
/// Later (observability phase), we will route the
/// OpenTelemetry export to Jaeger here—without modifying the services, which
/// simply call `telemetry::init(“auth”)`.
pub fn init(service_name: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(service = service_name, "telemetry initialized");
}
