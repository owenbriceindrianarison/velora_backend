use std::{sync::Arc, time::Duration};

use auth::{
    application::AuthUseCases,
    config,
    infrastructure::{self, Argon2Cipher, PasetoIssuer, PostgresUserRepository, RedisSessionStore},
    presentation::GrpcAuthService,
};
use velora_proto::auth::v1::auth_service_server::AuthServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    velora_common::telemetry::init("auth");
    let cfg = config::Config::load()?;

    // --- Connections (fail fast: the system crashes if the infrastructure is unavailable) ---
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;

    // Migrations embedded in the binary, applied at startup.
    sqlx::migrate!("./migrations").run(&pool).await?;

    let redis_client = redis::Client::open(cfg.redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;

    // NATS + JetStream
    let nats = async_nats::connect(&cfg.nats_url).await?;
    let jetstream = async_nats::jetstream::new(nats);
    jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "VELORA_USERS".to_string(),
            subjects: vec!["velora.users.>".to_string()],
            ..Default::default()
        })
        .await
        .map_err(|e| anyhow::anyhow!("Creating the NATS stream : {e}"))?;

    tokio::spawn(infrastructure::OutboxRelay::new(pool.clone(), jetstream).run());

    // --- Root composition: implementations are connected to the ports.
    // This is the only place in the service that knows everyone.
    let use_cases = Arc::new(AuthUseCases::new(
        Arc::new(PostgresUserRepository::new(pool)),
        Arc::new(Argon2Cipher),
        Arc::new(PasetoIssuer::from_hex(
            &cfg.paseto_secret_hex,
            Duration::from_secs(cfg.access_ttl_secs),
        )?),
        Arc::new(RedisSessionStore::new(redis_conn)),
        Duration::from_secs(cfg.refresh_ttl_secs),
    ));

    let addr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%addr, "auth-service listen (gRPC)");

    tonic::transport::Server::builder()
        .add_service(AuthServiceServer::new(GrpcAuthService::new(use_cases)))
        .serve(addr)
        .await?;

    Ok(())
}
