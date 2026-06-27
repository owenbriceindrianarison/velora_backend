use velora_common::config::{self, ConfigError};

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub grpc_port: u16,
    pub paseto_secret_hex: String,
    pub access_ttl_secs: u64,
    pub refresh_ttl_secs: u64,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: config::required("DATABASE_URL")?,
            redis_url: config::required("REDIS_URL")?,
            grpc_port: config::required_parse("GRPC_PORT")?,
            paseto_secret_hex: config::required("PASETO_SECRET_KEY")?,
            access_ttl_secs: config::optional("ACCESS_TTL_SECS", "900")
                .parse()
                .unwrap_or(900),
            refresh_ttl_secs: config::optional("REFRESH_TTL_SECS", "2592000")
                .parse()
                .unwrap_or(2_592_000), // 30 days
        })
    }
}
