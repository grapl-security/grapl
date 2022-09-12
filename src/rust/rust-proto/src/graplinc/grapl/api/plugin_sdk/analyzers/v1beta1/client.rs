use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};
use tracing::instrument;

use crate::{
    client_macros::RpcConfig,
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::messages as native,
    protobufs::graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::{
        self as proto,
        analyzer_service_client::AnalyzerServiceClient as AnalyzerServiceClientProto,
    },
    protocol::{
        endpoint::Endpoint,
        error::GrpcClientError,
        service_client::{
            ConnectError,
            Connectable,
        },
    },
};

#[derive(Clone)]
pub struct AnalyzerServiceClient {
    proto_client: AnalyzerServiceClientProto<tonic::transport::Channel>,
    executor: Executor,
}

#[async_trait::async_trait]
impl Connectable for AnalyzerServiceClient {
    const SERVICE_NAME: &'static str = "graplinc.grapl.api.plugin_registry.v1beta1.AnalyzerService";

    #[tracing::instrument(err)]
    async fn connect(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            AnalyzerServiceClientProto<tonic::transport::Channel>,
            endpoint,
        );

        Ok(AnalyzerServiceClient {
            proto_client,
            executor,
        })
    }
}

impl AnalyzerServiceClient {
    /// retrieve the plugin corresponding to the given plugin_id
    #[instrument(skip(self, request), err)]
    pub async fn run_analyzer(
        &mut self,
        request: native::RunAnalyzerRequest,
    ) -> Result<native::RunAnalyzerResponse, GrpcClientError> {
        execute_client_rpc!(
            self,
            request,
            run_analyzer,
            proto::RunAnalyzerRequest,
            native::RunAnalyzerResponse,
            RpcConfig::default(),
        )
    }
}
