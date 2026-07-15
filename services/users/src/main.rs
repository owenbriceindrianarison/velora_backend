use std::sync::Arc;

use velora_proto::users::v1::user_service_server::UserServiceServer;

use crate::{
    application::UserUseCases,
    infrasctruture::{PostgresAccountRepository, run_consumer},
    presentation::GrpcUserService,
};

mod application;
mod config;
mod domain;
mod infrasctruture;
mod presentation;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    velora_common::telemetry::init("users");
    let cfg = config::Config::load()?;

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let use_cases = Arc::new(UserUseCases::new(Arc::new(PostgresAccountRepository::new(
        pool,
    ))));

    // The NATS consumer and the gRPC server run IN PARALLEL:
    // This service has two entry points (events and requests).
    let nats = async_nats::connect(&cfg.nats_url).await?;
    let jetstream = async_nats::jetstream::new(nats);
    tokio::spawn({
        let use_cases = use_cases.clone();
        async move {
            if let Err(e) = run_consumer(jetstream, use_cases).await {
                tracing::error!(error = ?e, "NATS consumer stopped");
            }
        }
    });

    let addr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%addr, "users-service listening (gRPC)");

    tonic::transport::Server::builder()
        .add_service(UserServiceServer::new(GrpcUserService::new(use_cases)))
        .serve(addr)
        .await?;

    Ok(())
}
