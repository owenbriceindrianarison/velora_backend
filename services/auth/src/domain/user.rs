use time::OffsetDateTime;
use uuid::Uuid;

use super::{Email, HashedPassword};

#[derive(Debug, Clone)]
pub struct User {
    id: Uuid,
    email: Email,
    password_hash: HashedPassword,
    created_at: OffsetDateTime,
}

impl User {
    pub fn register(email: Email, password_hash: HashedPassword) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn from_persisted(
        id: Uuid,
        email: Email,
        password_hash: HashedPassword,
        created_at: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            email,
            password_hash,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn email(&self) -> &Email {
        &self.email
    }
    pub fn password_hash(&self) -> &HashedPassword {
        &self.password_hash
    }
    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}
