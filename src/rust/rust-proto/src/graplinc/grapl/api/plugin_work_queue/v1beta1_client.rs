use std::time::Duration;

use client_executor::{
    Executor,
    ExecutorConfig,
};
use proto::plugin_work_queue_service_client::PluginWorkQueueServiceClient as PluginWorkQueueServiceClientProto;

use crate::{
    client_factory::services::PluginWorkQueueClientConfig,
    client_macros::RpcConfig,
    create_proto_client,
    execute_client_rpc,
    graplinc::grapl::api::plugin_work_queue::v1beta1 as native,
    protobufs::graplinc::grapl::api::plugin_work_queue::v1beta1 as proto,
    protocol::{
        endpoint::Endpoint,
        error::GrpcClientError,
        service_client::{
            ConfigConnectable,
            ConnectError,
            Connectable,
        },
    },
};

pub type PluginWorkQueueServiceClientError = GrpcClientError;

#[derive(Clone)]
pub struct PluginWorkQueueServiceClient {
    executor: Executor,
    proto_client: PluginWorkQueueServiceClientProto<tonic::transport::Channel>,
}

#[async_trait::async_trait]
impl Connectable for PluginWorkQueueServiceClient {
    const SERVICE_NAME: &'static str =
        "graplinc.grapl.api.plugin_work_queue.v1beta1.PluginWorkQueueService";

    #[tracing::instrument(err)]
    async fn connect_with_endpoint(endpoint: Endpoint) -> Result<Self, ConnectError> {
        let executor = Executor::new(ExecutorConfig::new(Duration::from_secs(30)));
        let proto_client = create_proto_client!(
            executor,
            PluginWorkQueueServiceClientProto<tonic::transport::Channel>,
            endpoint,
        );

        Ok(Self {
            executor,
            proto_client,
        })
    }
}

impl ConfigConnectable for PluginWorkQueueServiceClient {
    type Config = PluginWorkQueueClientConfig;
}

impl PluginWorkQueueServiceClient {
    /// Adds a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn push_execute_generator(
        &mut self,
        request: native::PushExecuteGeneratorRequest,
    ) -> Result<native::PushExecuteGeneratorResponse, PluginWorkQueueServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            push_execute_generator,
            proto::PushExecuteGeneratorRequest,
            native::PushExecuteGeneratorResponse,
            RpcConfig::default(),
        )
    }

    /// Adds a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn push_execute_analyzer(
        &mut self,
        request: native::PushExecuteAnalyzerRequest,
    ) -> Result<native::PushExecuteAnalyzerResponse, PluginWorkQueueServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            push_execute_analyzer,
            proto::PushExecuteAnalyzerRequest,
            native::PushExecuteAnalyzerResponse,
            RpcConfig::default(),
        )
    }

    /// Retrieves a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_execute_generator(
        &mut self,
        request: native::GetExecuteGeneratorRequest,
    ) -> Result<native::GetExecuteGeneratorResponse, PluginWorkQueueServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            get_execute_generator,
            proto::GetExecuteGeneratorRequest,
            native::GetExecuteGeneratorResponse,
            RpcConfig::default(),
        )
    }

    /// Retrieves a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_execute_analyzer(
        &mut self,
        request: native::GetExecuteAnalyzerRequest,
    ) -> Result<native::GetExecuteAnalyzerResponse, PluginWorkQueueServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            get_execute_analyzer,
            proto::GetExecuteAnalyzerRequest,
            native::GetExecuteAnalyzerResponse,
            RpcConfig::default(),
        )
    }

    /// Acknowledges the completion of a generator job
    #[tracing::instrument(skip(self, request), err)]
    pub async fn acknowledge_generator(
        &mut self,
        request: native::AcknowledgeGeneratorRequest,
    ) -> Result<native::AcknowledgeGeneratorResponse, PluginWorkQueueServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            acknowledge_generator,
            proto::AcknowledgeGeneratorRequest,
            native::AcknowledgeGeneratorResponse,
            RpcConfig::default(),
        )
    }

    /// Acknowledges the completion of an analyzer job
    #[tracing::instrument(skip(self, request), err)]
    pub async fn acknowledge_analyzer(
        &mut self,
        request: native::AcknowledgeAnalyzerRequest,
    ) -> Result<native::AcknowledgeAnalyzerResponse, PluginWorkQueueServiceClientError> {
        execute_client_rpc!(
            self,
            request,
            acknowledge_analyzer,
            proto::AcknowledgeAnalyzerRequest,
            native::AcknowledgeAnalyzerResponse,
            RpcConfig::default(),
        )
    }
}
