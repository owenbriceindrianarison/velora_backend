use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("invalid email address")]
    InvalidEmail,

    #[error("invalid password: {0}")]
    WeakPassword(&'static str),
}
