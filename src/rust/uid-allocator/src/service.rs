use rust_proto_new::graplinc::grapl::api::uid_allocator::v1beta1::{
    messages::{
        AllocateIdsRequest,
        AllocateIdsResponse,
    },
    server::UidAllocatorApi,
};
use tonic::{
    async_trait,
    Code,
    Status,
};

use crate::allocator::UidAllocator;

pub struct UidAllocatorService {
    allocator: UidAllocator,
}

#[derive(thiserror::Error, Debug)]
pub enum UidAllocatorServiceError {
    #[error("Database error {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Unknown Tenant: {0}")]
    UnknownTenant(uuid::Uuid),
}

impl From<UidAllocatorServiceError> for Status {
    fn from(err: UidAllocatorServiceError) -> Self {
        match err {
            UidAllocatorServiceError::SqlxError(err) => {
                Status::new(Code::Internal, format!("Internal database error: {}", err))
            }
            UidAllocatorServiceError::UnknownTenant(tenant_id) => {
                Status::new(Code::NotFound, format!("Unknown Tenant: {}", tenant_id))
            }
        }
    }
}

#[async_trait]
impl UidAllocatorApi for UidAllocatorService {
    type Error = UidAllocatorServiceError;

    async fn allocate_ids(
        &self,
        request: AllocateIdsRequest,
    ) -> Result<AllocateIdsResponse, Self::Error> {
        let AllocateIdsRequest { count, tenant_id } = request;
        // `0` is a sentinel for "let the server decide on the allocation size"
        let count = if count == 0 { 1_000 } else { count };
        let allocation = self.allocator.allocate(tenant_id, count).await?;
        Ok(AllocateIdsResponse { allocation })
    }
}
