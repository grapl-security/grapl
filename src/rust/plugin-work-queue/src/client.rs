#![allow(warnings)]

use rust_proto::plugin_work_queue::{
    plugin_work_queue_service_client::PluginWorkQueueServiceClient as _PluginWorkQueueServiceClient,
    AcknowledgeRequest,
    AcknowledgeResponse,
    GetExecuteAnalyzerRequest,
    GetExecuteAnalyzerResponse,
    GetExecuteGeneratorRequest,
    GetExecuteGeneratorResponse,
    PutExecuteAnalyzerRequest,
    PutExecuteAnalyzerResponse,
    PutExecuteGeneratorRequest,
    PutExecuteGeneratorResponse,
    _AcknowledgeRequest,
    _AcknowledgeResponse,
    _GetExecuteAnalyzerRequest,
    _GetExecuteAnalyzerResponse,
    _GetExecuteGeneratorRequest,
    _GetExecuteGeneratorResponse,
    _PutExecuteAnalyzerRequest,
    _PutExecuteAnalyzerResponse,
    _PutExecuteGeneratorRequest,
    _PutExecuteGeneratorResponse,
};
use tonic::codegen::{
    Body,
    StdError,
};

#[derive(Debug, thiserror::Error)]
enum PluginWorkQueueServiceClientError {}

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
            .put_execute_generator(_PutExecuteGeneratorRequest::from(request))
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
            .put_execute_analyzer(_PutExecuteAnalyzerRequest::from(request))
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
            .get_execute_generator(_GetExecuteGeneratorRequest::from(request))
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
            .get_execute_analyzer(_GetExecuteAnalyzerRequest::from(request))
            .await
            .expect("todo");
        todo!()
    }

    /// Acknowledges the completion of a job
    pub async fn acknowledge(
        &mut self,
        request: AcknowledgeRequest,
    ) -> Result<AcknowledgeResponse, PluginWorkQueueServiceClientError> {
        self.inner
            .acknowledge(_AcknowledgeRequest::from(request))
            .await
            .expect("todo");
        todo!()
    }
}
