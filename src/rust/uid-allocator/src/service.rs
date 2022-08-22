use std::time::Duration;

use rust_proto::{
    graplinc::grapl::api::uid_allocator::v1beta1::{
        messages::{
            AllocateIdsRequest,
            AllocateIdsResponse,
            CreateTenantKeyspaceRequest,
            CreateTenantKeyspaceResponse,
        },
        server::{
            UidAllocatorApi,
            UidAllocatorServer,
        },
    },
    protocol::{
        healthcheck::HealthcheckStatus,
        status::Status,
    },
};
use sqlx::PgPool;

use crate::{
    allocator::UidAllocator,
    config::UidAllocatorServiceConfig,
};

pub struct UidAllocatorService {
    pub allocator: UidAllocator,
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
                Status::internal(format!("Internal database error: {}", err))
            }
            UidAllocatorServiceError::UnknownTenant(tenant_id) => {
                Status::not_found(format!("Unknown Tenant: {}", tenant_id))
            }
        }
    }
}

#[async_trait::async_trait]
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

    async fn create_tenant_keyspace(
        &self,
        request: CreateTenantKeyspaceRequest,
    ) -> Result<CreateTenantKeyspaceResponse, Self::Error> {
        let tenant_id = request.tenant_id;
        sqlx::query!(
            r"INSERT INTO counters (tenant_id, counter) VALUES ($1, 1)
            ON CONFLICT DO NOTHING;",
            tenant_id
        )
        .execute(&self.allocator.db.pool)
        .await?;

        // Create the entry

        Ok(CreateTenantKeyspaceResponse {})
    }
}

pub async fn exec_service(
    config: UidAllocatorServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = config.uid_allocator_bind_address;
    let pool = PgPool::connect(&config.counter_db_config.to_postgres_url()).await?;

    let allocator = UidAllocator::new(
        pool,
        config.preallocation_size,
        config.maximum_allocation_size,
        config.default_allocation_size,
    );
    let allocator = UidAllocatorService { allocator };

    let healthcheck_polling_interval_ms = 5000; // TODO: un-hardcode
    let (server, _shutdown_tx) = UidAllocatorServer::new(
        allocator,
        tokio::net::TcpListener::bind(addr.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );
    tracing::info!(
        message = "starting gRPC server",
        socket_address = %addr,
    );

    Ok(server.serve().await?)
}
