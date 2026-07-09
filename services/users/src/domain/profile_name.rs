use super::DomainError;

#[derive(Debug, Clone, Eq)]
pub struct ProfileName(String);

impl PartialEq for ProfileName {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl ProfileName {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidProfileName("cannot be empty"));
        }
        if trimmed.chars().count() > 30 {
            return Err(DomainError::InvalidProfileName("30 characters maximum"));
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn from_trusted(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
