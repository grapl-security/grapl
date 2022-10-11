use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};
use tonic::transport::Endpoint;

use crate::{
    client_factory::services::GraphQueryProxyClientConfig,
    client_macros::RpcConfig,
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::graph_query_proxy::v1beta1::messages as native,
    protobufs::graplinc::grapl::api::graph_query_proxy::v1beta1::{
        self as proto,
        graph_query_proxy_service_client::GraphQueryProxyServiceClient,
    },
    protocol::{
        error::GrpcClientError,
        service_client::{
            ConnectError,
            Connectable,
        },
    },
};
pub type GraphQueryProxyClientError = GrpcClientError;

#[derive(Clone)]
pub struct GraphQueryProxyClient {
    proto_client: GraphQueryProxyServiceClient<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for GraphQueryProxyClient {
    type Config = GraphQueryProxyClientConfig;
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.graph_query_proxy.v1beta1.GraphQueryProxyService";

    #[tracing::instrument(err)]
    async fn connect_with_endpoint(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            GraphQueryProxyServiceClient<tonic::transport::Channel>,
            endpoint,
        );

        Ok(Self {
            proto_client,
            executor,
        })
    }
}

impl GraphQueryProxyClient {
    pub async fn query_graph_with_uid(
        &mut self,
        request: native::QueryGraphWithUidRequest,
    ) -> Result<native::QueryGraphWithUidResponse, GraphQueryProxyClientError> {
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
    ) -> Result<native::QueryGraphFromUidResponse, GraphQueryProxyClientError> {
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
