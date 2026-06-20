use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    Missing(String),

    #[error("variable {0} invalid {1}")]
    Invalid(String, String),
}

/// Reads a required environment variable.
/// The service will refuse to start if it is missing: this is by design.
pub fn required(key: &str) -> Result<String, ConfigError> {
    std::env::var(key).map_err(|_| ConfigError::Missing(key.to_string()))
}

/// Reads an optional variable with a default value.
pub fn optional(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Reads and parses a variable into any type (u16 port, etc.).
pub fn required_parse<T>(key: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    required(key)?
        .parse()
        .map_err(|e: T::Err| ConfigError::Invalid(key.to_string(), e.to_string()))
}
