use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    application::{UserError, UserUseCases},
    domain::{Profile, ProfileKind},
};

use velora_proto::users::v1::{
    CreateProfileRequest, DeleteProfileRequest, DeleteProfileResponse, ListProfilesRequest,
    Profile as ProtoProfile, ProfileList, user_service_server::UserService,
};
pub struct GrpcUserService {
    use_cases: Arc<UserUseCases>,
}

impl GrpcUserService {
    pub fn new(use_cases: Arc<UserUseCases>) -> Self {
        Self { use_cases }
    }
}

fn to_status(err: UserError) -> Status {
    match err {
        UserError::Domain(e) => Status::invalid_argument(e.to_string()),
        UserError::AccountNotFound => Status::not_found(err.to_string()),
        UserError::Internal(e) => {
            tracing::error!(error = ?e, "users internal error");
            Status::internal("internal error")
        }
    }
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|_| Status::invalid_argument(format!("invalid {field}")))
}

fn to_proto(p: Profile) -> ProtoProfile {
    ProtoProfile {
        id: p.id.to_string(),
        name: p.name.as_str().to_string(),
        kids: matches!(p.kind, ProfileKind::Kids),
    }
}

#[tonic::async_trait]
impl UserService for GrpcUserService {
    async fn list_profiles(
        &self,
        req: Request<ListProfilesRequest>,
    ) -> Result<Response<ProfileList>, Status> {
        let account_id = parse_uuid(&req.into_inner().account_id, "account_id")?;
        let profiles = self
            .use_cases
            .list_profiles(account_id)
            .await
            .map_err(to_status)?;

        Ok(Response::new(ProfileList {
            profiles: profiles.into_iter().map(to_proto).collect(),
        }))
    }

    async fn create_profile(
        &self,
        req: Request<CreateProfileRequest>,
    ) -> Result<Response<ProtoProfile>, Status> {
        let r = req.into_inner();
        let account_id = parse_uuid(&r.account_id, "account_id")?;
        let profile = self
            .use_cases
            .create_profile(account_id, &r.name, r.kids)
            .await
            .map_err(to_status)?;

        Ok(Response::new(to_proto(profile)))
    }

    async fn delete_profile(
        &self,
        req: Request<DeleteProfileRequest>,
    ) -> Result<Response<DeleteProfileResponse>, Status> {
        let r = req.into_inner();
        let account_id = parse_uuid(&r.account_id, "account_id")?;
        let profile_id = parse_uuid(&r.profile_id, "profile_id")?;
        self.use_cases
            .delete_profile(account_id, profile_id)
            .await
            .map_err(to_status)?;

        Ok(Response::new(DeleteProfileResponse {}))
    }
}
