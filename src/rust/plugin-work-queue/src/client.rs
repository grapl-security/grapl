use rust_proto::plugin_work_queue::{
    plugin_work_queue_service_client::PluginWorkQueueServiceClient as _PluginWorkQueueServiceClient,
    AcknowledgeAnalyzerRequest,
    AcknowledgeAnalyzerRequestProto,
    AcknowledgeAnalyzerResponse,
    AcknowledgeGeneratorRequest,
    AcknowledgeGeneratorRequestProto,
    AcknowledgeGeneratorResponse,
    GetExecuteAnalyzerRequest,
    GetExecuteAnalyzerRequestProto,
    GetExecuteAnalyzerResponse,
    GetExecuteGeneratorRequest,
    GetExecuteGeneratorRequestProto,
    GetExecuteGeneratorResponse,
    PutExecuteAnalyzerRequest,
    PutExecuteAnalyzerRequestProto,
    PutExecuteAnalyzerResponse,
    PutExecuteGeneratorRequest,
    PutExecuteGeneratorRequestProto,
    PutExecuteGeneratorResponse,
};
use tonic::codegen::{
    Body,
    StdError,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueServiceClientError {}

pub struct PluginWorkQueueServiceClient<T> {
    inner: _PluginWorkQueueServiceClient<T>,
}

impl<T> PluginWorkQueueServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: Body + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(inner: _PluginWorkQueueServiceClient<T>) -> Self {
        Self { inner }
    }

    /// Adds a new execution job for a generator
    pub async fn put_execute_generator(
        &mut self,
        request: PutExecuteGeneratorRequest,
    ) -> Result<PutExecuteGeneratorResponse, PluginWorkQueueServiceClientError> {
        self.inner
            .put_execute_generator(PutExecuteGeneratorRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }

    /// Adds a new execution job for an analyzer
    pub async fn put_execute_analyzer(
        &mut self,
        request: PutExecuteAnalyzerRequest,
    ) -> Result<PutExecuteAnalyzerResponse, PluginWorkQueueServiceClientError> {
        self.inner
            .put_execute_analyzer(PutExecuteAnalyzerRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }

    /// Retrieves a new execution job for a generator
    pub async fn get_execute_generator(
        &mut self,
        request: GetExecuteGeneratorRequest,
    ) -> Result<GetExecuteGeneratorResponse, PluginWorkQueueServiceClientError> {
        self.inner
            .get_execute_generator(GetExecuteGeneratorRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }

    /// Retrieves a new execution job for an analyzer
    pub async fn get_execute_analyzer(
        &mut self,
        request: GetExecuteAnalyzerRequest,
    ) -> Result<GetExecuteAnalyzerResponse, PluginWorkQueueServiceClientError> {
        self.inner
            .get_execute_analyzer(GetExecuteAnalyzerRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }

    /// Acknowledges the completion of a generator job
    pub async fn acknowledge_generator(
        &mut self,
        request: AcknowledgeGeneratorRequest,
    ) -> Result<AcknowledgeGeneratorResponse, PluginWorkQueueServiceClientError> {
        self.inner
            .acknowledge_generator(AcknowledgeGeneratorRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }

    /// Acknowledges the completion of a job
    pub async fn acknowledge_analyzer(
        &mut self,
        request: AcknowledgeAnalyzerRequest,
    ) -> Result<AcknowledgeAnalyzerResponse, PluginWorkQueueServiceClientError> {
        self.inner
            .acknowledge_analyzer(AcknowledgeAnalyzerRequestProto::from(request))
            .await
            .expect("todo");
        todo!()
    }
}
