use thiserror::Error;

use crate::domain::DomainError;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("email already taken")]
    EmailTaken,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("expired or revoked session")]
    SessionNotFound,

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}
