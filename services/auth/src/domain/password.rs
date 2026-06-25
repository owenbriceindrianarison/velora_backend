use crate::domain::DomainError;

#[derive(Clone)]
pub struct RawPassword(String);

impl RawPassword {
    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.len() < 8 {
            return Err(DomainError::WeakPassword(
                "password too short (8 characters min)",
            ));
        }
        if value.len() > 128 {
            return Err(DomainError::WeakPassword(
                "password too long (128 characters max)",
            ));
        }
        Ok(Self(value))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

/// No #[derive(Debug)]: we implement it manually so that an accidental log (`{:?}`) NEVER displays the password.
impl std::fmt::Display for RawPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RawPassword(***)")
    }
}

#[derive(Debug, Clone)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub fn from_trusted(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
