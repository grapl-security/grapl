use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};
use tonic::transport::Endpoint;

use crate::{
    client_macros::RpcConfig,
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::graph_query_service::v1beta1::messages as native,
    protobufs::graplinc::grapl::api::graph_query_service::v1beta1::{
        self as proto,
        graph_query_service_client::GraphQueryServiceClient,
    },
    protocol::{
        error::GrpcClientError,
        service_client::{
            ConnectError,
            Connectable,
        },
    },
};
pub type GraphQueryClientError = GrpcClientError;

#[derive(Clone)]
pub struct GraphQueryClient {
    proto_client: GraphQueryServiceClient<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for GraphQueryClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.graph_query_service.v1beta1.GraphQueryService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            GraphQueryServiceClient<tonic::transport::Channel>,
            endpoint,
        );

        Ok(Self {
            proto_client,
            executor,
        })
    }
}

impl GraphQueryClient {
    pub async fn query_graph_with_uid(
        &mut self,
        request: native::QueryGraphWithUidRequest,
    ) -> Result<native::QueryGraphWithUidResponse, GraphQueryClientError> {
        execute_client_rpc!(
            self,
            request,
            query_graph_with_uid,
            proto::QueryGraphWithUidRequest,
            native::QueryGraphWithUidResponse,
            RpcConfig::default(),
        )
    }
    pub async fn query_graph_from_uid(
        &mut self,
        request: native::QueryGraphFromUidRequest,
    ) -> Result<native::QueryGraphFromUidResponse, GraphQueryClientError> {
        execute_client_rpc!(
            self,
            request,
            query_graph_from_uid,
            proto::QueryGraphFromUidRequest,
            native::QueryGraphFromUidResponse,
            RpcConfig::default(),
        )
    }
}
