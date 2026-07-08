use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::{
    application::{OutboxMessage, RepositoryError, UserRespository},
    domain::{Email, HashedPassword, User},
};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: uuid::Uuid,
    email: String,
    password_hash: String,
    created_at: OffsetDateTime,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User::from_persisted(
            row.id,
            Email::from_trusted(row.email),
            HashedPassword::from_trusted(row.password_hash),
            row.created_at,
        )
    }
}

#[async_trait]
impl UserRespository for PostgresUserRepository {
    async fn insert(&self, user: &User, event: OutboxMessage) -> Result<(), RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Other(e.into()))?;

        sqlx::query(
            "INSERT INTO users(id, email, password_hash, created_at)
            VALUES ($1, $2, $3, $4)",
        )
        .bind(user.id())
        .bind(user.email().as_str())
        .bind(user.password_hash().as_str())
        .bind(user.created_at())
        .execute(&mut *tx)
        .await
        .map_err(|e| match &e {
            // 23505 = Postgres uniqueness violation: the email address already exists.
            sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => {
                RepositoryError::Conflict
            }
            _ => RepositoryError::Other(e.into()),
        })?;

        sqlx::query(
            "INSERT INTO outbox (id, subject, payload)
            VALUES ($1, $2, $3)",
        )
        .bind(event.id)
        .bind(event.subject)
        .bind(event.payload)
        .execute(&mut *tx)
        .await
        .map_err(|e| RepositoryError::Other(e.into()))?;

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Other(e.into()))?;

        Ok(())
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, email, password_hash, created_at
            FROM users WHERE email = $1
            ",
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Other(e.into()))?;

        Ok(row.map(User::from))
    }
}
