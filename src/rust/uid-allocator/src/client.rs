use dashmap::DashMap;
pub use rust_proto::graplinc::grapl::api::uid_allocator::v1beta1::client::UidAllocatorServiceClient;
use rust_proto::{
    client_factory::services::UidAllocatorClientConfig,
    graplinc::grapl::api::uid_allocator::v1beta1::{
        client::UidAllocatorServiceClientError,
        messages::{
            AllocateIdsRequest,
            Allocation,
            CreateTenantKeyspaceRequest,
        },
    },
    protocol::service_client::{
        ConnectError,
        ConnectWithConfig,
    },
};

#[derive(Clone)]
pub struct CachingUidAllocatorServiceClient {
    pub allocator: UidAllocatorServiceClient,
    pub allocation_map: DashMap<uuid::Uuid, Allocation>,
    /// The number of ids to request
    pub count: u32,
}

impl CachingUidAllocatorServiceClient {
    pub fn new(allocator: UidAllocatorServiceClient, count: u32) -> Self {
        Self {
            allocator,
            allocation_map: DashMap::with_capacity(1),
            count,
        }
    }

    pub async fn from_client_config(
        client_config: UidAllocatorClientConfig,
        count: u32,
    ) -> Result<Self, ConnectError> {
        let allocator = UidAllocatorServiceClient::connect_with_config(client_config).await?;
        Ok(Self::new(allocator, count))
    }

    pub async fn allocate_id(
        &self,
        tenant_id: uuid::Uuid,
    ) -> Result<u64, UidAllocatorServiceClientError> {
        match self.get_from_allocation_map(tenant_id) {
            Some(allocation) => Ok(allocation),
            None => {
                let mut allocator = self.allocator.clone();
                let mut allocation = allocator
                    .allocate_ids(AllocateIdsRequest {
                        tenant_id: tenant_id.into(),
                        count: self.count,
                    })
                    .await?
                    .allocation;
                let next = allocation.next().unwrap(); // Allocation should never be empty
                self.allocation_map.insert(tenant_id, allocation);
                Ok(next)
            }
        }
    }

    pub async fn create_tenant_keyspace(
        &mut self,
        request: CreateTenantKeyspaceRequest,
    ) -> Result<(), UidAllocatorServiceClientError> {
        self.allocator.create_tenant_keyspace(request).await?;
        Ok(())
    }

    fn get_from_allocation_map(&self, tenant_id: uuid::Uuid) -> Option<u64> {
        if let Some(mut allocation) = self.allocation_map.get_mut(&tenant_id) {
            allocation.next()
        } else {
            None
        }
    }
}

impl std::fmt::Debug for CachingUidAllocatorServiceClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("CachingUidAllocatorServiceClient");
        for entry in self.allocation_map.iter() {
            d.field("tenant_id", entry.key());
        }
        d.field("count", &self.count).finish()
    }
}
