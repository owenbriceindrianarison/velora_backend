use std::sync::Arc;

use tonic::{metadata::MetadataValue, Request, Response, Status};
use velora_proto::auth::v1::{
    auth_service_server::AuthService, AuthTokens, LoginRequest, LogoutRequest, LogoutResponse,
    RefreshSessionRequest, RegisterRequest,
};

use crate::application::{use_cases::TokenPair, AuthError, AuthUseCases};

pub struct GrpcAuthService {
    use_cases: Arc<AuthUseCases>,
}

impl GrpcAuthService {
    pub fn new(use_cases: Arc<AuthUseCases>) -> Self {
        Self { use_cases }
    }
}

/// Translation of business errors into gRPC statuses.
fn to_status(err: AuthError) -> Status {
    tracing::info!(err = %err, "debug error");

    fn with_code(mut status: Status, code: &'static str) -> Status {
        status
            .metadata_mut()
            .insert("x-error-code", MetadataValue::from_static(code));
        status
    }

    let code = err.error_code();
    match err {
        AuthError::Domain(e) => with_code(Status::invalid_argument(e.to_string()), code),
        AuthError::EmailTaken => with_code(Status::already_exists("email already taken"), code),
        AuthError::InvalidCredentials => {
            with_code(Status::unauthenticated("invalid credentials"), code)
        }
        AuthError::SessionNotFound => {
            with_code(Status::unauthenticated("expired or revoked session"), code)
        }
        AuthError::Internal(e) => {
            tracing::error!(error = ?e, "auth internal error");
            Status::internal("internal error")
        }
    }
}

fn to_proto(pair: TokenPair) -> AuthTokens {
    AuthTokens {
        access_token: pair.access.token,
        refresh_token: pair.refresh,
        expires_in: pair.access.expires_in_secs,
    }
}

#[tonic::async_trait]
impl AuthService for GrpcAuthService {
    async fn register(
        &self,
        req: Request<RegisterRequest>,
    ) -> Result<Response<AuthTokens>, Status> {
        let r = req.into_inner();
        let pair = self
            .use_cases
            .register(&r.email, &r.password)
            .await
            .map_err(to_status)?;

        Ok(Response::new(to_proto(pair)))
    }

    async fn login(&self, req: Request<LoginRequest>) -> Result<Response<AuthTokens>, Status> {
        let r = req.into_inner();
        let pair = self
            .use_cases
            .login(&r.email, &r.password)
            .await
            .map_err(to_status)?;

        Ok(Response::new(to_proto(pair)))
    }

    async fn refresh_session(
        &self,
        req: Request<RefreshSessionRequest>,
    ) -> Result<Response<AuthTokens>, Status> {
        let r = req.into_inner();
        let pair = self
            .use_cases
            .refresh(&r.refresh_token)
            .await
            .map_err(to_status)?;

        Ok(Response::new(to_proto(pair)))
    }

    async fn logout(
        &self,
        req: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        self.use_cases
            .logout(&req.into_inner().refresh_token)
            .await
            .map_err(to_status)?;

        Ok(Response::new(LogoutResponse {}))
    }
}
