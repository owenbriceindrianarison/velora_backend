use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("profile name invalid : {0}")]
    InvalidProfileName(&'static str),

    #[error("5-profile limit reached")]
    TooManyProfiles,

    #[error("a profile already has that name")]
    DuplicateProfileName,

    #[error("unable to delete the last profile")]
    LastProfile,

    #[error("profile not found")]
    ProfileNotFound,
}
