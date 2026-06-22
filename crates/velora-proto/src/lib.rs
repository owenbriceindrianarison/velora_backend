//! gRPC code generated from /proto.
//! Each service imports this crate: it is the single,
//! typed contract for all inter-service communication.

pub mod auth {
    pub mod v1 {
        tonic::include_proto!("velora.auth.v1");
    }
}
