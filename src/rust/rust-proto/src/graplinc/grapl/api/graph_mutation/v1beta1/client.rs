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
        graph_mutation::v1beta1::messages as native,
        protocol::{
            error::GrpcClientError,
            service_client::{
                ConnectError,
                Connectable,
            },
        },
    },
    protobufs::graplinc::grapl::api::graph_mutation::v1beta1::{
        self as proto,
        graph_mutation_service_client::GraphMutationServiceClient,
    },
};
pub type GraphMutationClientError = GrpcClientError;

#[derive(Clone)]
pub struct GraphMutationClient {
    proto_client: GraphMutationServiceClient<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for GraphMutationClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.graph_mutation.v1beta1.GraphMutationService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            GraphMutationServiceClient<tonic::transport::Channel>,
            endpoint,
        );

        Ok(Self {
            proto_client,
            executor,
        })
    }
}

impl GraphMutationClient {
    pub async fn create_node(
        &mut self,
        request: native::CreateNodeRequest,
    ) -> Result<native::CreateNodeResponse, GraphMutationClientError> {
        execute_client_rpc!(
            self,
            request,
            create_node,
            proto::CreateNodeRequest,
            native::CreateNodeResponse,
            RpcConfig::default(),
        )
    }

    pub async fn set_node_property(
        &mut self,
        request: native::SetNodePropertyRequest,
    ) -> Result<native::SetNodePropertyResponse, GraphMutationClientError> {
        execute_client_rpc!(
            self,
            request,
            set_node_property,
            proto::SetNodePropertyRequest,
            native::SetNodePropertyResponse,
            RpcConfig::default(),
        )
    }

    pub async fn create_edge(
        &mut self,
        request: native::CreateEdgeRequest,
    ) -> Result<native::CreateEdgeResponse, GraphMutationClientError> {
        execute_client_rpc!(
            self,
            request,
            create_edge,
            proto::CreateEdgeRequest,
            native::CreateEdgeResponse,
            RpcConfig::default(),
        )
    }
}
