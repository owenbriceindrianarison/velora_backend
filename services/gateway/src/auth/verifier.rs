use pasetors::{
    claims::ClaimsValidationRules, keys::AsymmetricPublicKey, public, token::UntrustedToken,
    version4::V4, Public,
};
use uuid::Uuid;

/// Verifies PASETO v4.public using only the PUBLIC key.
/// The gateway can verify, but will NEVER be able to sign.
pub struct PasetoVerifier {
    public_key: AsymmetricPublicKey<V4>,
}

impl PasetoVerifier {
    pub fn from_hex(public_hex: &str) -> Result<Self, anyhow::Error> {
        let bytes = hex::decode(public_hex)?;
        let public_key = AsymmetricPublicKey::<V4>::from(&bytes)
            .map_err(|e| anyhow::anyhow!("invalid PASETO's public key : {e}"))?;
        Ok(Self { public_key })
    }

    /// Returns the user ID if the token is authentic and valid.
    /// pasetors automatically checks for expiration (the `exp` claim).
    pub fn verify(&self, token: &str) -> Option<Uuid> {
        let untrusted = UntrustedToken::<Public, V4>::try_from(token).ok()?;

        let mut rules = ClaimsValidationRules::new();
        rules.validate_issuer_with("velora-auth"); // Rejects all other senders

        let trusted = public::verify(&self.public_key, &untrusted, &rules, None, None).ok()?;

        let sub = trusted.payload_claims()?.get_claim("sub")?.as_str()?;
        Uuid::parse_str(sub).ok()
    }
}
