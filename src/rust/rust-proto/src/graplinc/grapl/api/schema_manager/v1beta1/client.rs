use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};
use tonic::transport::Endpoint;

use crate::{
    client_macros::ExecuteClientRpcOptions,
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::schema_manager::v1beta1::messages as native,
    protobufs::graplinc::grapl::api::schema_manager::{
        v1beta1 as proto,
        v1beta1::schema_manager_service_client::SchemaManagerServiceClient as SchemaManagerServiceClientProto,
    },
    protocol::{
        error::GrpcClientError,
        service_client::{
            ConnectError,
            Connectable,
        },
    },
};

pub type SchemaManagerClientError = GrpcClientError;

#[derive(Clone)]
pub struct SchemaManagerClient {
    proto_client: SchemaManagerServiceClientProto<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for SchemaManagerClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.schema_manager.v1beta1.SchemaManagerService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            SchemaManagerServiceClientProto<tonic::transport::Channel>,
            endpoint,
        );

        Ok(Self {
            proto_client,
            executor,
        })
    }
}

impl SchemaManagerClient {
    pub async fn deploy_schema(
        &mut self,
        request: native::DeploySchemaRequest,
    ) -> Result<native::DeploySchemaResponse, SchemaManagerClientError> {
        execute_client_rpc!(
            self,
            request,
            deploy_schema,
            proto::DeploySchemaRequest,
            native::DeploySchemaResponse,
            ExecuteClientRpcOptions::default(),
        )
    }

    pub async fn get_edge_schema(
        &mut self,
        request: native::GetEdgeSchemaRequest,
    ) -> Result<native::GetEdgeSchemaResponse, SchemaManagerClientError> {
        execute_client_rpc!(
            self,
            request,
            get_edge_schema,
            proto::GetEdgeSchemaRequest,
            native::GetEdgeSchemaResponse,
            ExecuteClientRpcOptions::default(),
        )
    }
}
