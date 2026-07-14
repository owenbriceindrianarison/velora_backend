use thiserror::Error;

use crate::domain::DomainError;

#[derive(Debug, Error)]
pub enum UserError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("account not found")]
    AccountNotFound,

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}
