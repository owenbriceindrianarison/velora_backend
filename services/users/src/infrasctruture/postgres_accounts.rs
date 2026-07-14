use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    application::AccountRepository,
    domain::{Account, Profile, ProfileKind, ProfileName},
};

pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn kind_to_db(kind: ProfileKind) -> &'static str {
    match kind {
        ProfileKind::Adult => "adult",
        ProfileKind::Kids => "kids",
    }
}

#[derive(sqlx::FromRow)]
struct ProfileRow {
    id: Uuid,
    name: String,
    kind: String,
}

impl From<ProfileRow> for Profile {
    fn from(row: ProfileRow) -> Self {
        Profile {
            id: row.id,
            name: ProfileName::from_trusted(row.name),
            kind: if row.kind == "kids" {
                ProfileKind::Kids
            } else {
                ProfileKind::Adult
            },
        }
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn insert_account(&self, account: &Account) -> Result<(), anyhow::Error> {
        let mut tx = self.pool.begin().await?;

        // ON CONFLICT DO NOTHING = consumer idempotence.
        // When the event is replayed twice, it does not create anything the second time.
        let inserted = sqlx::query(
            "INSERT INTO accounts (id, email) VALUES ($1, $2)
            ON CONFLICT (id) DO NOTHING",
        )
        .bind(account.id())
        .bind(account.email())
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if inserted > 0 {
            for profile in account.profiles() {
                sqlx::query(
                    "INSERT INTO profiles (id, account_id, name, kind)
                VALUES ($1, $2, $3, $4)",
                )
                .bind(profile.id)
                .bind(account.id())
                .bind(profile.name.as_str())
                .bind(kind_to_db(profile.kind))
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find(&self, account_id: Uuid) -> Result<Option<Account>, anyhow::Error> {
        //  let exists: Option<(String,)> =
        //     sqlx::query_as("SELECT email FROM accounts WHERE id = $1")
        //         .bind(account_id)
        //         .fetch_optional(&self.pool)
        //         .await?;

        // let Some((email,)) = exists else { return Ok(None) };
        let email: Option<String> = sqlx::query_scalar("SELECT email FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(email) = email else {
            return Ok(None);
        };

        let profiles: Vec<ProfileRow> = sqlx::query_as(
            "SELECT id, name, kind, FROM profiles
            WHERE account_id = $1 ORDER BY created_at",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(Account::from_persistence(
            account_id,
            email,
            profiles.into_iter().map(Profile::from).collect(),
        )))
    }

    async fn insert_profile(
        &self,
        account_id: Uuid,
        profile: &Profile,
    ) -> Result<(), anyhow::Error> {
        sqlx::query("INSERT INTO profiles (id, account_id, name, kind) VALUES ($1, $2, $3, $4)")
            .bind(profile.id)
            .bind(account_id)
            .bind(profile.name.as_str())
            .bind(kind_to_db(profile.kind))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_profile(
        &self,
        account_id: Uuid,
        profile_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM profile WHEN id = $1 AND account_id = $2")
            .bind(profile_id)
            .bind(account_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
