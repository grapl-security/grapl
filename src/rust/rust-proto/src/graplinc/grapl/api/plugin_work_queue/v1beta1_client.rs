use tonic::transport::Endpoint;

use crate::{
    graplinc::grapl::api::{
        client::{
            Client,
            ClientError,
            Connectable,
            WithClient,
        },
        plugin_work_queue::v1beta1 as native,
    },
    protobufs::graplinc::grapl::api::plugin_work_queue::v1beta1::plugin_work_queue_service_client::PluginWorkQueueServiceClient,
};

#[async_trait::async_trait]
impl Connectable for PluginWorkQueueServiceClient<tonic::transport::Channel> {
    async fn connect(endpoint: Endpoint) -> Result<Self, ClientError> {
        Ok(Self::connect(endpoint).await?)
    }
}

#[derive(Clone)]
pub struct PluginWorkQueueClient {
    client: Client<PluginWorkQueueServiceClient<tonic::transport::Channel>>,
}

impl WithClient<PluginWorkQueueServiceClient<tonic::transport::Channel>> for PluginWorkQueueClient {
    fn with_client(
        client: Client<PluginWorkQueueServiceClient<tonic::transport::Channel>>,
    ) -> Self {
        Self { client }
    }
}

impl PluginWorkQueueClient {
    /// Adds a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn push_execute_generator(
        &mut self,
        request: native::PushExecuteGeneratorRequest,
    ) -> Result<native::PushExecuteGeneratorResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.push_execute_generator(request).await },
            )
            .await
    }

    /// Adds a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn push_execute_analyzer(
        &mut self,
        request: native::PushExecuteAnalyzerRequest,
    ) -> Result<native::PushExecuteAnalyzerResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.push_execute_analyzer(request).await },
            )
            .await
    }

    /// Retrieves a new execution job for a generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_execute_generator(
        &mut self,
        request: native::GetExecuteGeneratorRequest,
    ) -> Result<native::GetExecuteGeneratorResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.get_execute_generator(request).await },
            )
            .await
    }

    /// Retrieves a new execution job for an analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn get_execute_analyzer(
        &mut self,
        request: native::GetExecuteAnalyzerRequest,
    ) -> Result<native::GetExecuteAnalyzerResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.get_execute_analyzer(request).await },
            )
            .await
    }

    /// Acknowledges the completion of a generator job
    #[tracing::instrument(skip(self, request), err)]
    pub async fn acknowledge_generator(
        &mut self,
        request: native::AcknowledgeGeneratorRequest,
    ) -> Result<native::AcknowledgeGeneratorResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.acknowledge_generator(request).await },
            )
            .await
    }

    /// Acknowledges the completion of an analyzer job
    #[tracing::instrument(skip(self, request), err)]
    pub async fn acknowledge_analyzer(
        &mut self,
        request: native::AcknowledgeAnalyzerRequest,
    ) -> Result<native::AcknowledgeAnalyzerResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.acknowledge_analyzer(request).await },
            )
            .await
    }

    /// Retrieve the current queue depth for the given generator
    #[tracing::instrument(skip(self, request), err)]
    pub async fn queue_depth_for_generator(
        &mut self,
        request: native::QueueDepthForGeneratorRequest,
    ) -> Result<native::QueueDepthForGeneratorResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.queue_depth_for_generator(request).await },
            )
            .await
    }

    /// Retrieve the current queue depth for the given analyzer
    #[tracing::instrument(skip(self, request), err)]
    pub async fn queue_depth_for_analyzer(
        &mut self,
        request: native::QueueDepthForAnalyzerRequest,
    ) -> Result<native::QueueDepthForAnalyzerResponse, ClientError> {
        self.client
            .execute(
                request,
                None,
                |status| status.code() == tonic::Code::Unavailable,
                10,
                |mut client, request| async move { client.queue_depth_for_analyzer(request).await },
            )
            .await
    }
}
