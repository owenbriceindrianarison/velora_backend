use velora_common::config::{self, ConfigError};

pub struct Config {
    pub http_port: u16,
    pub auth_grpc_url: String,
    pub paseto_public_hex: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            http_port: config::required_parse("HTTP_PORT")?,
            auth_grpc_url: config::required("AUTH_GRPC_URL")?,
            paseto_public_hex: config::required("PASETO_PUBLIC_KEY")?,
        })
    }
}
