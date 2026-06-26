use std::time::Duration;

use anyhow::Ok;
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use uuid::Uuid;

use crate::application::SessionStore;

pub struct RedisSessionStore {
    conn: ConnectionManager,
}

impl RedisSessionStore {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }

    pub fn key(refresh_hash: &str) -> String {
        format!("auth:refresh:{refresh_hash}")
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn store(
        &self,
        refresh_hash: &str,
        user_id: uuid::Uuid,
        ttl: Duration,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.conn.clone();
        // SET key user_id EX ttl: Redis handles expiration on its own;
        // an expired refresh token is automatically removed without any cron job.
        redis::cmd("SET")
            .arg(Self::key(refresh_hash))
            .arg(user_id.to_string())
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn consume(&self, refresh_hash: &str) -> Result<Option<Uuid>, anyhow::Error> {
        let mut conn = self.conn.clone();
        // GETDEL: read + delete in a SINGLE atomic operation.
        // Two concurrent requests with the same token cannot both succeed.
        let value: Option<String> = redis::cmd("GETDEL")
            .arg(Self::key(refresh_hash))
            .query_async(&mut conn)
            .await?;

        Ok(value.and_then(|v| Uuid::parse_str(&v).ok()))
    }
}
