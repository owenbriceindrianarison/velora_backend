use std::{sync::Arc, time::Duration};

use auth::{
    application::{AuthError, AuthUseCases},
    domain::DomainError,
    infrastructure::{Argon2Cipher, PasetoIssuer, PostgresUserRepository, RedisSessionStore},
};
use sqlx::PgPool;

async fn build(pool: PgPool) -> AuthUseCases {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let paseto_key = std::env::var("PASETO_SECRET_KEY").expect("PASETO_SECRET_KEY not set");

    let redis_client = redis::Client::open(redis_url.as_str()).unwrap();
    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .unwrap();

    AuthUseCases::new(
        Arc::new(PostgresUserRepository::new(pool)),
        Arc::new(Argon2Cipher),
        Arc::new(PasetoIssuer::from_hex(&paseto_key, Duration::from_secs(900)).unwrap()),
        Arc::new(RedisSessionStore::new(redis_conn)),
        Duration::from_secs(86_400),
    )
}

fn unique_email() -> String {
    format!("test_{}@velora.test", uuid::Uuid::new_v4())
}

fn unwrap_err<T>(res: Result<T, AuthError>) -> AuthError {
    match res {
        Err(e) => e,
        Ok(_) => panic!("expected Err, got Ok"),
    }
}

// === Register =================================================================

#[sqlx::test(migrations = "./migrations")]
async fn register_returns_access_and_refresh_tokens(pool: PgPool) {
    let uc = build(pool).await;
    let pair = uc.register(&unique_email(), "StrongPass1!").await.unwrap();
    assert!(pair.access.token.starts_with("v4.public."));
    assert!(!pair.refresh.is_empty());
    assert!(pair.access.expires_in_secs > 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_duplicate_email_is_rejected(pool: PgPool) {
    let uc = build(pool).await;
    let email = unique_email();
    uc.register(&email, "StrongPass1!").await.unwrap();
    let err = unwrap_err(uc.register(&email, "AnotherPass2!").await);
    assert!(matches!(err, AuthError::EmailTaken));
}

#[sqlx::test(migrations = "./migrations")]
async fn register_rejects_malformed_email(pool: PgPool) {
    let uc = build(pool).await;
    let err = unwrap_err(uc.register("not-an-email", "StrongPass1!").await);
    assert!(matches!(err, AuthError::Domain(DomainError::InvalidEmail)));
}

#[sqlx::test(migrations = "./migrations")]
async fn register_rejects_short_password(pool: PgPool) {
    let uc = build(pool).await;
    let err = unwrap_err(uc.register(&unique_email(), "short").await);
    assert!(matches!(err, AuthError::Domain(DomainError::WeakPassword(_))));
}

// === Login ====================================================================

#[sqlx::test(migrations = "./migrations")]
async fn login_with_correct_credentials_returns_tokens(pool: PgPool) {
    let uc = build(pool).await;
    let email = unique_email();
    uc.register(&email, "StrongPass1!").await.unwrap();

    let pair = uc.login(&email, "StrongPass1!").await.unwrap();
    assert!(pair.access.token.starts_with("v4.public."));
    assert!(!pair.refresh.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn login_wrong_password_is_rejected(pool: PgPool) {
    let uc = build(pool).await;
    let email = unique_email();
    uc.register(&email, "StrongPass1!").await.unwrap();

    let err = unwrap_err(uc.login(&email, "WrongPassword!").await);
    assert!(matches!(err, AuthError::InvalidCredentials));
}

#[sqlx::test(migrations = "./migrations")]
async fn login_unknown_email_is_rejected(pool: PgPool) {
    let uc = build(pool).await;
    let err = unwrap_err(uc.login(&unique_email(), "StrongPass1!").await);
    assert!(matches!(err, AuthError::InvalidCredentials));
}

// === Refresh ==================================================================

#[sqlx::test(migrations = "./migrations")]
async fn refresh_returns_new_tokens(pool: PgPool) {
    let uc = build(pool).await;
    let first = uc.register(&unique_email(), "StrongPass1!").await.unwrap();
    let second = uc.refresh(&first.refresh).await.unwrap();
    assert!(second.access.token.starts_with("v4.public."));
    assert!(!second.refresh.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn refresh_rotates_token_and_revokes_the_old_one(pool: PgPool) {
    let uc = build(pool).await;
    let first = uc.register(&unique_email(), "StrongPass1!").await.unwrap();
    let second = uc.refresh(&first.refresh).await.unwrap();

    assert_ne!(second.refresh, first.refresh);

    let err = unwrap_err(uc.refresh(&first.refresh).await);
    assert!(matches!(err, AuthError::SessionNotFound));
}

#[sqlx::test(migrations = "./migrations")]
async fn refresh_with_invalid_token_is_rejected(pool: PgPool) {
    let uc = build(pool).await;
    let err = unwrap_err(uc.refresh("invalid-token").await);
    assert!(matches!(err, AuthError::SessionNotFound));
}

// === Logout ===================================================================

#[sqlx::test(migrations = "./migrations")]
async fn logout_invalidates_refresh_token(pool: PgPool) {
    let uc = build(pool).await;
    let pair = uc.register(&unique_email(), "StrongPass1!").await.unwrap();

    uc.logout(&pair.refresh).await.unwrap();

    let err = unwrap_err(uc.refresh(&pair.refresh).await);
    assert!(matches!(err, AuthError::SessionNotFound));
}

#[sqlx::test(migrations = "./migrations")]
async fn logout_with_invalid_token_does_not_error(pool: PgPool) {
    let uc = build(pool).await;
    uc.logout("invalid-token").await.unwrap();
}
