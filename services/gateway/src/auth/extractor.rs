use axum::{extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

/// Axum extractor: adding `CurrentUser` as a parameter to a handler
/// is enough to make the route protected. No middleware to set up, no chance of forgetting—the type system handles the protection.
pub struct CurrentUser(pub Uuid);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;

        let user_id = state.paseto.verify(token).ok_or(ApiError::Unauthorized)?;

        Ok(CurrentUser(user_id))
    }
}
