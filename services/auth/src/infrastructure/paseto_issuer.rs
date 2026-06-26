use pasetors::{claims::Claims, keys::AsymmetricSecretKey, public, version4::V4};

use crate::application::{AccessToken, TokenIssuer};

/// PASETO v4.public: ASYMMETRIC Ed25519 signature.
/// The auth-service holds the PRIVATE key (it signs).
/// The gateway will only have the PUBLIC key (it verifies).
/// Compromising the gateway therefore does NOT allow tokens to be forged.
pub struct PasetoIssuer {
    secret: AsymmetricSecretKey<V4>,
    ttl: std::time::Duration,
}

impl PasetoIssuer {
    pub fn from_hex(secret_hex: &str, ttl: std::time::Duration) -> Result<Self, anyhow::Error> {
        let bytes = hex::decode(secret_hex)?;
        let secret = AsymmetricSecretKey::<V4>::from(&bytes)
            .map_err(|e| anyhow::anyhow!("invalid PASETO key : {e}"))?;

        Ok(Self { secret, ttl })
    }
}

impl TokenIssuer for PasetoIssuer {
    fn issue(&self, user_id: uuid::Uuid) -> Result<crate::application::AccessToken, anyhow::Error> {
        let mut claims =
            Claims::new_expires_in(&self.ttl).map_err(|e| anyhow::anyhow!("claims : {e}"))?;
        claims
            .issuer("velora-auth")
            .and_then(|_| claims.subject(&user_id.to_string()))
            .map_err(|e| anyhow::anyhow!("claims : {e}"))?;

        let token = public::sign(&self.secret, &claims, None, None)
            .map_err(|e| anyhow::anyhow!("signature PASETO : {e}"))?;

        Ok(AccessToken {
            token,
            expires_in_secs: self.ttl.as_secs() as i64,
        })
    }
}
