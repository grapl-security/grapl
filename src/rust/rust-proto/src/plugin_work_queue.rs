#![allow(warnings)]
pub use crate::graplinc::grapl::api::plugin_work_queue::v1beta1::{
    plugin_work_queue_service_client,
    plugin_work_queue_service_server,
    AcknowledgeRequest as _AcknowledgeRequest,
    AcknowledgeResponse as _AcknowledgeResponse,
    ExecutionJob as _ExecutionJob,
    GetExecuteAnalyzerRequest as _GetExecuteAnalyzerRequest,
    GetExecuteAnalyzerResponse as _GetExecuteAnalyzerResponse,
    GetExecuteGeneratorRequest as _GetExecuteGeneratorRequest,
    GetExecuteGeneratorResponse as _GetExecuteGeneratorResponse,
    PutExecuteAnalyzerRequest as _PutExecuteAnalyzerRequest,
    PutExecuteAnalyzerResponse as _PutExecuteAnalyzerResponse,
    PutExecuteGeneratorRequest as _PutExecuteGeneratorRequest,
    PutExecuteGeneratorResponse as _PutExecuteGeneratorResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueDeserializationError {
    #[error("Missing a required field")]
    MissingRequiredField(&'static str),
    #[error("Empty field")]
    EmptyField(&'static str),
}

pub struct ExecutionJob {
    tenant_id: uuid::Uuid,
    plugin_id: uuid::Uuid,
    data: Vec<u8>,
}

impl TryFrom<_ExecutionJob> for ExecutionJob {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: _ExecutionJob) -> Result<Self, Self::Error> {
        let tenant_id = value
            .tenant_id
            .ok_or(Self::Error::MissingRequiredField("ExecutionJob.tenant_id"))?
            .into();
        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingRequiredField("ExecutionJob.plugin_id"))?
            .into();

        let data = value.data;
        if data.is_empty() {
            return Err(Self::Error::EmptyField("ExecutionJob.data"));
        }
        Ok(Self {
            tenant_id,
            plugin_id,
            data,
        })
    }
}

impl From<ExecutionJob> for _ExecutionJob {
    fn from(value: ExecutionJob) -> Self {
        assert!(!value.data.is_empty());

        Self {
            tenant_id: Some(value.tenant_id.into()),
            plugin_id: Some(value.plugin_id.into()),
            data: value.data,
        }
    }
}

pub struct AcknowledgeRequest {
    request_id: uuid::Uuid,
}

impl TryFrom<_AcknowledgeRequest> for AcknowledgeRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: _AcknowledgeRequest) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id
            .ok_or(Self::Error::MissingRequiredField(
                "AcknowledgeRequest.request_id",
            ))?
            .into();
        Ok(Self { request_id })
    }
}

impl From<AcknowledgeRequest> for _AcknowledgeRequest {
    fn from(value: AcknowledgeRequest) -> Self {
        Self {
            request_id: Some(value.request_id.into()),
        }
    }
}

pub struct AcknowledgeResponse {}

impl TryFrom<_AcknowledgeResponse> for AcknowledgeResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: _AcknowledgeResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<AcknowledgeResponse> for _AcknowledgeResponse {
    fn from(_value: AcknowledgeResponse) -> Self {
        Self {}
    }
}

pub struct GetExecuteAnalyzerRequest {}

impl TryFrom<_GetExecuteAnalyzerRequest> for GetExecuteAnalyzerRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: _GetExecuteAnalyzerRequest) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<GetExecuteAnalyzerRequest> for _GetExecuteAnalyzerRequest {
    fn from(_value: GetExecuteAnalyzerRequest) -> Self {
        Self {}
    }
}

pub struct GetExecuteAnalyzerResponse {
    execution_job: ExecutionJob,
    request_id: uuid::Uuid,
}

impl TryFrom<_GetExecuteAnalyzerResponse> for GetExecuteAnalyzerResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: _GetExecuteAnalyzerResponse) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id
            .ok_or(Self::Error::MissingRequiredField(
                "GetExecuteAnalyzerResponse.request_id",
            ))?
            .into();
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "GetExecuteAnalyzerResponse.execution_job",
            ))?
            .try_into()?;
        Ok(Self {
            request_id,
            execution_job,
        })
    }
}

impl From<GetExecuteAnalyzerResponse> for _GetExecuteAnalyzerResponse {
    fn from(value: GetExecuteAnalyzerResponse) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
            request_id: Some(value.request_id.into()),
        }
    }
}

pub struct GetExecuteGeneratorRequest {}

impl TryFrom<_GetExecuteGeneratorRequest> for GetExecuteGeneratorRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: _GetExecuteGeneratorRequest) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<GetExecuteGeneratorRequest> for _GetExecuteGeneratorRequest {
    fn from(_value: GetExecuteGeneratorRequest) -> Self {
        Self {}
    }
}

pub struct GetExecuteGeneratorResponse {
    execution_job: ExecutionJob,
    request_id: uuid::Uuid,
}

impl TryFrom<_GetExecuteGeneratorResponse> for GetExecuteGeneratorResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: _GetExecuteGeneratorResponse) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id
            .ok_or(Self::Error::MissingRequiredField(
                "GetExecuteGeneratorResponse.request_id",
            ))?
            .into();
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "GetExecuteGeneratorResponse.execution_job",
            ))?
            .try_into()?;
        Ok(Self {
            request_id,
            execution_job,
        })
    }
}

impl From<GetExecuteGeneratorResponse> for _GetExecuteGeneratorResponse {
    fn from(value: GetExecuteGeneratorResponse) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
            request_id: Some(value.request_id.into()),
        }
    }
}

pub struct PutExecuteAnalyzerRequest {
    execution_job: ExecutionJob,
}

impl TryFrom<_PutExecuteAnalyzerRequest> for PutExecuteAnalyzerRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: _PutExecuteAnalyzerRequest) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "PutExecuteAnalyzerRequest.execution_job",
            ))?
            .try_into()?;
        Ok(Self { execution_job })
    }
}

impl From<PutExecuteAnalyzerRequest> for _PutExecuteAnalyzerRequest {
    fn from(value: PutExecuteAnalyzerRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
        }
    }
}

pub struct PutExecuteAnalyzerResponse {}

impl TryFrom<_PutExecuteAnalyzerResponse> for PutExecuteAnalyzerResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: _PutExecuteAnalyzerResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PutExecuteAnalyzerResponse> for _PutExecuteAnalyzerResponse {
    fn from(_value: PutExecuteAnalyzerResponse) -> Self {
        Self {}
    }
}

pub struct PutExecuteGeneratorRequest {
    execution_job: ExecutionJob,
}

impl TryFrom<_PutExecuteGeneratorRequest> for PutExecuteGeneratorRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: _PutExecuteGeneratorRequest) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "PutExecuteGeneratorRequest.execution_job",
            ))?
            .try_into()?;
        Ok(Self { execution_job })
    }
}

impl From<PutExecuteGeneratorRequest> for _PutExecuteGeneratorRequest {
    fn from(value: PutExecuteGeneratorRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
        }
    }
}

pub struct PutExecuteGeneratorResponse {}

impl TryFrom<_PutExecuteGeneratorResponse> for PutExecuteGeneratorResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: _PutExecuteGeneratorResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PutExecuteGeneratorResponse> for _PutExecuteGeneratorResponse {
    fn from(_value: PutExecuteGeneratorResponse) -> Self {
        Self {}
    }
}
