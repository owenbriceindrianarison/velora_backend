use std::sync::Arc;

use futures::StreamExt;
use velora_events::users::{SUBJECT_REGISTERED, UserRegistered};

use crate::application::UserUsecases;

pub async fn run_consumer(
    jetstream: async_nats::jetstream::Context,
    use_cases: Arc<UserUsecases>,
) -> anyhow::Result<()> {
    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "VELORA_USERS".to_string(),
            subjects: vec!["velora.users.>".to_string()],
            ..Default::default()
        })
        .await
        .map_err(|e| anyhow::anyhow!("stream NATS : {e}"))?;

    let consumer = stream
        .get_or_create_consumer(
            "users-service",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("users-service".to_string()),
                filter_subject: SUBJECT_REGISTERED.to_string(), // "velora.users.registered"
                ..Default::default()
            },
        )
        .await
        .map_err(|e| anyhow::anyhow!("consumer NATS : {e}"))?;

    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| anyhow::anyhow!("message flow : {e}"))?;

    tracing::info!("NATS consumer ready (velora.users.registered)");

    while let Some(message) = messages.next().await {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                tracing::error!(error = %e, "NATS reception");
                continue;
            }
        };

        match serde_json::from_slice::<UserRegistered>(&message.payload) {
            Ok(event) => {
                match use_cases
                    .on_user_registered(event.user_id, event.email)
                    .await
                {
                    // ACK only if successful. Without an ACK, JetStream will resend
                    Ok(()) => {
                        if let Err(e) = message.ack().await {
                            tracing::error!(error = %e, "ack NATS");
                        }
                    }
                    Err(e) => tracing::error!(error = ?e, "user.registered method"),
                }
            }
            // Unreadable payload: ACK anyway; resending it indefinitely won't make it any more readable (poison message).
            Err(e) => {
                tracing::error!(error = %e, "Invalid payload, abandoned");
                let _ = message.ack().await;
            }
        }
    }

    Ok(())
}
