//! The outbox relay: a background loop that flushes the outbox table to NATS.

use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

pub struct OutboxRelay {
    pool: PgPool,
    jetstream: async_nats::jetstream::Context,
}

#[derive(sqlx::FromRow)]
struct PendingEvent {
    id: Uuid,
    subject: String,
    payload: serde_json::Value,
}

impl OutboxRelay {
    pub fn new(pool: PgPool, jetstream: async_nats::jetstream::Context) -> Self {
        Self { pool, jetstream }
    }

    /// À lancer avec tokio::spawn : tourne pour toute la vie du service
    pub async fn run(self) {
        loop {
            if let Err(e) = self.drain_batch().await {
                tracing::error!(error = ?e, "Outbox relay: batch failed");
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    async fn drain_batch(&self) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // FOR UPDATE SKIP LOCKED: if we scale to multiple instances of the auth service,
        // each relay will process DIFFERENT rows without blocking each other. Free scalability.
        let pending: Vec<PendingEvent> = sqlx::query_as(
            "SELECT id, subject, payload FROM outbox
            WHERE published_at IS NULL
            ORDER BY created_at
            LIMIT 50
            FOR UPDATE SKIP LOCKED",
        )
        .fetch_all(&mut *tx)
        .await?;

        if pending.is_empty() {
            return Ok(());
        }

        let mut published_ids = Vec::with_capacity(pending.len());
        for event in &pending {
            let bytes = serde_json::to_vec(&event.payload)?;
            // Double await: the first sends, the second waits for the ACK from JetStream—proof that the event has been PERSISTED.
            self.jetstream
                .publish(event.subject.clone(), bytes.into())
                .await?
                .await?;
            published_ids.push(event.id);
        }

        sqlx::query("UPDATE outbox SET published_at = now() WHERE id = ANY($1)")
            .bind(&published_ids)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        tracing::debug!(count = published_ids.len(), "events forwarded to NATS");
        Ok(())
    }
}
