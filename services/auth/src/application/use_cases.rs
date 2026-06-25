use std::{sync::Arc, time::Duration};

use rand::RngCore;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    application::RepositoryError,
    domain::{Email, RawPassword, User},
};

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

    pub async fn register(&self, email: &str, password: String) -> Result<TokenPair, AuthError> {
        let email = Email::parse(email)?;
        let password = RawPassword::parse(password)?;

        let hash = self.cipher.hash(&password)?;

        let user = User::register(email, hash);

        self.users.insert(&user).await.map_err(|e| match e {
            RepositoryError::Conflict => AuthError::EmailTaken,
            RepositoryError::Other(e) => AuthError::Internal(e),
        })?;

        tracing::info!(user_id = %user.id(), "new user registered");

        self.open_session(user.id()).await
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
