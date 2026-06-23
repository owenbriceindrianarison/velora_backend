use super::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        let normalized = value.trim().to_lowercase();
        let well_formed = normalized.len() > 2
            && normalized.len() < 255
            && normalized.matches("@").count() == 1
            && !normalized.starts_with("@")
            && !normalized.ends_with("@")
            && !normalized.contains(char::is_whitespace);

        if well_formed {
            Ok(Self(normalized))
        } else {
            Err(DomainError::InvalidEmail)
        }
    }

    pub fn from_trusted(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_email() {
        let email = Email::parse(" BriceOwen@Velora.com ").unwrap();
        assert_eq!(email.as_str(), "briceowen@velora.com");
    }

    #[test]
    fn invalid_email() {
        for bad in ["", " brice", "@velora.com", "a@@b@c.com", "a b@c.com"] {
            assert_eq!(Email::parse(bad), Err(DomainError::InvalidEmail));
        }
    }
}
