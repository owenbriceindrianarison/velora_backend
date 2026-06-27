//! One-time utility: generates the Ed25519 key pair for PASETO.
//! Usage: cargo run -p auth --bin genkeys
//! Copy the two lines to .env.dev (NEVER commit to production).

use pasetors::keys::{AsymmetricKeyPair, Generate};
use pasetors::version4::V4;

fn main() {
    let kp = AsymmetricKeyPair::<V4>::generate().expect("Ed25519 generation");
    println!("PASETO_SECRET_KEY={}", hex::encode(kp.secret.as_bytes()));
    println!("PASETO_PUBLIC_KEY={}", hex::encode(kp.public.as_bytes()));
}
