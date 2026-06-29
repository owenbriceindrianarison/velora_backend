use std::sync::Arc;

use tonic::transport::Channel;
use velora_proto::auth::v1::auth_service_client::AuthServiceClient;

use crate::auth::PasetoVerifier;

/// State shared among all handlers. Cloned per request:
/// Everything it contains must be inexpensive to clone
/// (Arc, and Tonic clients that are handles on a Channel).
#[derive(Clone)]
pub struct AppState {
    pub auth_client: AuthServiceClient<Channel>,
    pub paseto: Arc<PasetoVerifier>,
}

impl AppState {
    pub fn new(auth_grpc_url: String, paseto: PasetoVerifier) -> Result<Self, anyhow::Error> {
        // connect_lazy: The connection is established on the FIRST request.
        // The gateway therefore starts even if auth is still compiling—essential with cargo-watch,
        // where each service restarts at its own pace.
        let channel = Channel::from_shared(auth_grpc_url)?.connect_lazy();

        Ok(Self {
            auth_client: AuthServiceClient::new(channel),
            paseto: Arc::new(paseto),
        })
    }
}
