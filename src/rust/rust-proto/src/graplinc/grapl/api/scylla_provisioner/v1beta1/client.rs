use std::time::Duration;

use client_executor::{Executor, ExecutorConfig};
use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::scylla_provisioner::v1beta1::messages::{
        self as native,
    },
    protobufs::graplinc::grapl::api::scylla_provisioner::v1beta1::{
        scylla_provisioner_service_client::ScyllaProvisionerServiceClient,
        self as proto,
    },
    protocol::{error::GrpcClientError, service_client::{Connectable, ConnectError}},
    create_proto_client, execute_client_rpc, client_macros::ExecuteClientRpcOptions,
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
    pub async fn deploy_graph_schemas(
        &mut self,
        request: native::DeployGraphSchemasRequest,
    ) -> Result<native::DeployGraphSchemasResponse, ScyllaProvisionerClientError> {
        execute_client_rpc!(
            self,
            request,
            deploy_graph_schemas,
            proto::DeployGraphSchemasRequest,
            native::DeployGraphSchemasResponse,
            ExecuteClientRpcOptions::default(),
        )
    }
}
