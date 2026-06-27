use std::sync::Arc;

use tonic::{Request, Response, Status};
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
    match err {
        AuthError::Domain(e) => Status::invalid_argument(e.to_string()),
        AuthError::EmailTaken => Status::already_exists(err.to_string()),
        AuthError::InvalidCredentials => Status::unauthenticated(err.to_string()),
        AuthError::SessionNotFound => Status::unauthenticated(err.to_string()),
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
