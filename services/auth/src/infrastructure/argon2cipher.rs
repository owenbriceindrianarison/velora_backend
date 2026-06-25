use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

use crate::{
    application::PasswordCipher,
    domain::{HashedPassword, RawPassword},
};

/// Argon2id: the recommended algorithm (OWASP) for passwords.
/// Slow BY DESIGN — that's what makes brute-force attacks prohibitively expensive.
pub struct Argon2Cipher;

impl PasswordCipher for Argon2Cipher {
    fn hash(&self, raw: &RawPassword) -> Result<HashedPassword, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(raw.expose().as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("argon2 hash : {e}"))?;

        Ok(HashedPassword::from_trusted(hash.to_string()))
    }

    fn verify(&self, raw: &RawPassword, hash: &HashedPassword) -> Result<bool, anyhow::Error> {
        let parsed = PasswordHash::new(hash.as_str())
            .map_err(|e| anyhow::anyhow!("corrupted hash : {e}"))?;

        Ok(Argon2::default()
            .verify_password(raw.expose().as_bytes(), &parsed)
            .is_ok())
    }
}
