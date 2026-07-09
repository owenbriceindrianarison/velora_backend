use uuid::Uuid;

use super::{DomainError, ProfileName};

const MAX_PROFILES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileKind {
    Adult,
    Kids,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub id: Uuid,
    pub name: ProfileName,
    pub kind: ProfileKind,
}

#[derive(Debug)]
pub struct Account {
    id: Uuid,
    email: String,
    profiles: Vec<Profile>,
}

impl Account {
    /// Creation triggered by the user.registered event
    pub fn create(id: Uuid, email: String) -> Self {
        let default_profile = Profile {
            id: Uuid::new_v4(),
            name: ProfileName::from_trusted("Main Profile".to_string()),
            kind: ProfileKind::Adult,
        };
        Self {
            id,
            email,
            profiles: vec![default_profile],
        }
    }

    pub fn from_persistence(id: Uuid, email: String, profiles: Vec<Profile>) -> Self {
        Self {
            id,
            email,
            profiles,
        }
    }

    pub fn add_profile(
        &mut self,
        name: ProfileName,
        kind: ProfileKind,
    ) -> Result<Profile, DomainError> {
        if self.profiles.len() >= MAX_PROFILES {
            return Err(DomainError::TooManyProfiles);
        }
        if self.profiles.iter().any(|p| p.name == name) {
            return Err(DomainError::DuplicateProfileName);
        }

        let profile = Profile {
            id: Uuid::new_v4(),
            name,
            kind,
        };
        self.profiles.push(profile.clone());
        Ok(profile)
    }

    pub fn remove_profile(&mut self, profile_id: Uuid) -> Result<(), DomainError> {
        if self.profiles.len() == 1 {
            return Err(DomainError::LastProfile);
        }
        let before = self.profiles.len();
        self.profiles.retain(|p| p.id != profile_id);

        if self.profiles.len() == before {
            return Err(DomainError::ProfileNotFound);
        }
        Ok(())
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(s: &str) -> ProfileName {
        ProfileName::parse(s).unwrap()
    }

    #[test]
    fn account_created_with_default_profile() {
        let account = Account::create(Uuid::new_v4(), "a@b.com".into());
        assert_eq!(account.profiles.len(), 1);
    }

    #[test]
    fn refuse_sixth_profile() {
        let mut account = Account::create(Uuid::new_v4(), "a@b.com".into());
        for i in 1..5 {
            account
                .add_profile(name(&format!("P{i}")), ProfileKind::Adult)
                .unwrap();
        }
        let err = account
            .add_profile(name("P6"), ProfileKind::Adult)
            .unwrap_err();
        assert_eq!(err, DomainError::TooManyProfiles);
    }

    #[test]
    fn refuse_duplicate_names_and_last_profile() {
        let mut account = Account::create(Uuid::new_v4(), "a@b.com".into());
        let err = account
            .add_profile(name("Main profile"), ProfileKind::Kids)
            .unwrap_err();
        assert_eq!(err, DomainError::DuplicateProfileName);

        let only = account.profiles()[0].id;
        assert_eq!(
            account.remove_profile(only).unwrap_err(),
            DomainError::LastProfile
        );
    }
}
