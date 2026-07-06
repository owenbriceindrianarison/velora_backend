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

impl AuthError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Domain(e) => e.error_code(),
            Self::EmailTaken => "EMAIL_TAKEN",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::SessionNotFound => "SESSION_NOT_FOUND",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}
