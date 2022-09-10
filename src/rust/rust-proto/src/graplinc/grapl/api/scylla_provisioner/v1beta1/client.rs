use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};
use tonic::transport::Endpoint;

use crate::{
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::{
        client_macros::RpcConfig,
        protocol::{
            error::GrpcClientError,
            service_client::{
                ConnectError,
                Connectable,
            },
        },
        scylla_provisioner::v1beta1::messages::{self as native,},
    },
    protobufs::graplinc::grapl::api::scylla_provisioner::v1beta1::{
        self as proto,
        scylla_provisioner_service_client::ScyllaProvisionerServiceClient,
    },
};

pub type ScyllaProvisionerClientError = GrpcClientError;

pub struct ScyllaProvisionerClient {
    proto_client: ScyllaProvisionerServiceClient<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for ScyllaProvisionerClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.scylla_provisioner.v1beta1.ScyllaProvisionerService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            ScyllaProvisionerServiceClient<tonic::transport::Channel>,
            endpoint,
        );

        Ok(Self {
            proto_client,
            executor,
        })
    }
}

impl ScyllaProvisionerClient {
    pub async fn provision_graph_for_tenant(
        &mut self,
        request: native::ProvisionGraphForTenantRequest,
    ) -> Result<native::ProvisionGraphForTenantResponse, ScyllaProvisionerClientError> {
        execute_client_rpc!(
            self,
            request,
            provision_graph_for_tenant,
            proto::ProvisionGraphForTenantRequest,
            native::ProvisionGraphForTenantResponse,
            RpcConfig::default(),
        )
    }
}
