use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::{AccountRepository, UserError},
    domain::{Account, Profile, ProfileKind, ProfileName},
};

pub struct UserUseCases {
    accounts: Arc<dyn AccountRepository>,
}

impl UserUseCases {
    pub fn new(accounts: Arc<dyn AccountRepository>) -> Self {
        Self { accounts }
    }

    pub async fn on_user_registered(&self, user_id: Uuid, email: String) -> Result<(), UserError> {
        let account = Account::create(user_id, email);
        self.accounts.insert_account(&account).await?;

        tracing::info!(account_id = %user_id, "account created with default profile");

        Ok(())
    }

    pub async fn create_profile(
        &self,
        account_id: Uuid,
        name: &str,
        kids: bool,
    ) -> Result<Profile, UserError> {
        let name = ProfileName::parse(name)?;
        let kind = if kids {
            ProfileKind::Kids
        } else {
            ProfileKind::Adult
        };

        let mut account = self
            .accounts
            .find(account_id)
            .await?
            .ok_or(UserError::AccountNotFound)?;
        let profile = account.add_profile(name, kind)?;
        self.accounts.insert_profile(account_id, &profile).await?;

        Ok(profile)
    }

    pub async fn list_profiles(&self, account_id: Uuid) -> Result<Vec<Profile>, UserError> {
        let account = self
            .accounts
            .find(account_id)
            .await?
            .ok_or(UserError::AccountNotFound)?;

        Ok(account.profiles().to_vec())
    }

    pub async fn delete_profile(
        &self,
        account_id: Uuid,
        profile_id: Uuid,
    ) -> Result<(), UserError> {
        let mut account = self
            .accounts
            .find(account_id)
            .await?
            .ok_or(UserError::AccountNotFound)?;

        account.remove_profile(profile_id)?;
        self.accounts.delete_profile(account_id, profile_id).await?;

        Ok(())
    }
}
