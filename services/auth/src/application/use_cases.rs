use std::{sync::Arc, time::Duration};

use rand::RngCore;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use velora_events::users::{UserRegistered, SUBJECT_REGISTERED};

use crate::application::RepositoryError;
use crate::domain::{Email, RawPassword, User};

use super::OutboxMessage;

use super::{AccessToken, AuthError, PasswordCipher, SessionStore, TokenIssuer, UserRespository};

pub struct TokenPair {
    pub access: AccessToken,
    /// Sent in PLAIN TEXT to the client, only once. On the server side,
    /// only its hash exists.
    pub refresh: String,
}

pub struct AuthUseCases {
    users: Arc<dyn UserRespository>,
    cipher: Arc<dyn PasswordCipher>,
    tokens: Arc<dyn TokenIssuer>,
    sessions: Arc<dyn SessionStore>,
    refresh_ttl: Duration,
}

impl AuthUseCases {
    pub fn new(
        users: Arc<dyn UserRespository>,
        cipher: Arc<dyn PasswordCipher>,
        tokens: Arc<dyn TokenIssuer>,
        sessions: Arc<dyn SessionStore>,
        refresh_ttl: Duration,
    ) -> Self {
        Self {
            users,
            cipher,
            tokens,
            sessions,
            refresh_ttl,
        }
    }

    pub async fn register(&self, email: &str, password: &str) -> Result<TokenPair, AuthError> {
        let email = Email::parse(email)?;
        let password = RawPassword::parse(password)?;
        let hash = self.cipher.hash(&password)?;
        let user = User::register(email, hash);

        let event = OutboxMessage::new(
            SUBJECT_REGISTERED,
            serde_json::to_value(UserRegistered {
                user_id: user.id(),
                email: user.email().as_str().to_string(),
                occured_at: OffsetDateTime::now_utc(),
            })
            .map_err(|e| AuthError::Internal(e.into()))?,
        );

        self.users.insert(&user, event).await.map_err(|e| match e {
            RepositoryError::Conflict => AuthError::EmailTaken,
            RepositoryError::Other(e) => AuthError::Internal(e),
        })?;

        tracing::info!(user_id = %user.id(), "new user registered");

        self.open_session(user.id()).await
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<TokenPair, AuthError> {
        let email = Email::parse(email)?;
        let password = RawPassword::parse(password)?;

        let user = self
            .users
            .find_by_email(&email)
            .await
            .map_err(|e| AuthError::Internal(e.into()))?
            .ok_or(AuthError::InvalidCredentials)?;

        let valid = self.cipher.verify(&password, user.password_hash())?;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }

        self.open_session(user.id()).await
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<TokenPair, AuthError> {
        // consume = GETDEL: the old token is invalidated here, whether we succeed or not.
        // This is ROTATION: a token that is stolen and replayed after legitimate use will fail.
        let user_id = self
            .sessions
            .consume(&hash_token(refresh_token))
            .await?
            .ok_or(AuthError::SessionNotFound)?;

        self.open_session(user_id).await
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AuthError> {
        // Redeem without reissuing = revoke.
        self.sessions.consume(&hash_token(refresh_token)).await?;

        Ok(())
    }

    /// Emits an access/refresh pair and saves the session.
    async fn open_session(&self, user_id: Uuid) -> Result<TokenPair, AuthError> {
        let access = self.tokens.issue(user_id)?;
        let refresh = generate_refresh_token();

        self.sessions
            .store(&hash_token(&refresh), user_id, self.refresh_ttl)
            .await?;

        Ok(TokenPair { access, refresh })
    }
}

/// 256 bits of cryptographic randomness, encoded in hexadecimal.
fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
