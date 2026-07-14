use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Account, Profile};

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn insert_account(&self, account: &Account) -> Result<(), anyhow::Error>;

    async fn find(&self, account_id: Uuid) -> Result<Option<Account>, anyhow::Error>;

    async fn insert_profile(
        &self,
        account_id: Uuid,
        profile: &Profile,
    ) -> Result<(), anyhow::Error>;

    async fn delete_profile(&self, account_id: Uuid, profile_id: Uuid)
    -> Result<(), anyhow::Error>;
}
