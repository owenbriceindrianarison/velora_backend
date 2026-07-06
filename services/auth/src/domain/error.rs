use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("invalid email address")]
    InvalidEmail,

    #[error("invalid password: {0}")]
    WeakPassword(&'static str),
}

impl DomainError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidEmail => "INVALID_EMAIL",
            Self::WeakPassword(_) => "WEAK_PASSWORD",
        }
    }
}
