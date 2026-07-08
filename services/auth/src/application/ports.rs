use serde_json::Value;
use std::time::Duration;

/// PORTS: what the application needs, expressed in traits.
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Email, HashedPassword, RawPassword, User};

pub struct OutboxMessage {
    pub id: Uuid,
    pub subject: &'static str,
    pub payload: Value,
}

impl OutboxMessage {
    pub fn new(subject: &'static str, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            subject,
            payload,
        }
    }
}

#[async_trait]
pub trait UserRespository: Send + Sync {
    async fn insert(&self, user: &User, event: OutboxMessage) -> Result<(), RepositoryError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("uniqueness conflict")]
    Conflict,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Password hashing (implemented using Argon2 on the infrastructure side).
pub trait PasswordCipher: Send + Sync {
    fn hash(&self, raw: &RawPassword) -> Result<HashedPassword, anyhow::Error>;
    fn verify(&self, raw: &RawPassword, hash: &HashedPassword) -> Result<bool, anyhow::Error>;
}

/// Issuing access tokens (implemented by PASETO on the infrastructure side).
pub trait TokenIssuer: Send + Sync {
    fn issue(&self, user_id: Uuid) -> Result<AccessToken, anyhow::Error>;
}

pub struct AccessToken {
    pub token: String,
    pub expires_in_secs: i64,
}

/// Storage of refresh tokens (implemented by Redis on the infrastructure side).
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn store(
        &self,
        refresh_hash: &str,
        user_id: Uuid,
        ttl: Duration,
    ) -> Result<(), anyhow::Error>;

    /// Reads AND atomically deletes (GETDEL): this is what makes the rotation secure—a refresh token can only be used once
    async fn consume(&self, refresh_hash: &str) -> Result<Option<Uuid>, anyhow::Error>;
}
