use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod users {
    use super::*;

    pub const SUBJECT_REGISTERED: &str = "velora.users.registered";

    #[derive(Debug, Serialize, Deserialize)]
    pub struct UserRegistered {
        pub user_id: Uuid,
        pub email: String,
        #[serde(with = "time::serde::rfc3339")]
        pub occured_at: time::OffsetDateTime,
    }
}
