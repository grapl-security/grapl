#![allow(unused_variables)]

use std::fmt::Formatter;

use crate::protobufs::graplinc::grapl::api::plugin_work_queue::v1beta1 as proto;
use proto::{
    get_execute_analyzer_response,
    get_execute_generator_response,
};
use crate::{
    serde_impl::ProtobufSerializable,
    type_url,
    SerDeError,
};
/*
pub use crate::graplinc::grapl::api::plugin_work_queue::v1beta1::{
    plugin_work_queue_service_client,
    plugin_work_queue_service_server,
};
*/

pub struct ExecutionJob {
    pub tenant_id: uuid::Uuid,
    pub plugin_id: uuid::Uuid,
    pub data: Vec<u8>,
}

impl std::fmt::Debug for ExecutionJob {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionJob")
            .field("tenant_id", &self.tenant_id)
            .field("plugin_id", &self.plugin_id)
            .field("data.len", &self.data.len())
            .finish()
    }
}

impl TryFrom<proto::ExecutionJob> for ExecutionJob {
    type Error = SerDeError;

    fn try_from(value: proto::ExecutionJob) -> Result<Self, Self::Error> {
        let tenant_id = value
            .tenant_id
            .ok_or(Self::Error::MissingField("ExecutionJob.tenant_id"))?
            .into();
        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingField("ExecutionJob.plugin_id"))?
            .into();

        let data = value.data;
        if data.is_empty() {
            return Err(Self::Error::MissingField("ExecutionJob.data"));
        }
        Ok(Self {
            tenant_id,
            plugin_id,
            data,
        })
    }
}

impl From<ExecutionJob> for proto::ExecutionJob {
    fn from(value: ExecutionJob) -> Self {
        assert!(!value.data.is_empty());

        Self {
            tenant_id: Some(value.tenant_id.into()),
            plugin_id: Some(value.plugin_id.into()),
            data: value.data,
        }
    }
}

#[derive(Debug)]
pub struct AcknowledgeGeneratorRequest {
    pub request_id: i64,
    pub success: bool,
}

impl TryFrom<proto::AcknowledgeGeneratorRequest> for AcknowledgeGeneratorRequest {
    type Error = SerDeError;

    fn try_from(value: proto::AcknowledgeGeneratorRequest) -> Result<Self, Self::Error> {
        let request_id = value.request_id;
        let success = value.success;
        Ok(Self {
            request_id,
            success,
        })
    }
}

impl From<AcknowledgeGeneratorRequest> for proto::AcknowledgeGeneratorRequest {
    fn from(value: AcknowledgeGeneratorRequest) -> Self {
        Self {
            request_id: value.request_id,
            success: value.success,
        }
    }
}

#[derive(Debug)]
pub struct AcknowledgeGeneratorResponse {}

impl TryFrom<proto::AcknowledgeGeneratorResponse> for AcknowledgeGeneratorResponse {
    type Error = SerDeError;

    fn try_from(_value: proto::AcknowledgeGeneratorResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<AcknowledgeGeneratorResponse> for proto::AcknowledgeGeneratorResponse {
    fn from(_value: AcknowledgeGeneratorResponse) -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct AcknowledgeAnalyzerRequest {
    pub request_id: i64,
    pub success: bool,
}

impl TryFrom<proto::AcknowledgeAnalyzerRequest> for AcknowledgeAnalyzerRequest {
    type Error = SerDeError;

    fn try_from(value: proto::AcknowledgeAnalyzerRequest) -> Result<Self, Self::Error> {
        let request_id = value.request_id;

        let success = value.success;
        Ok(Self {
            request_id,
            success,
        })
    }
}

impl From<AcknowledgeAnalyzerRequest> for proto::AcknowledgeAnalyzerRequest {
    fn from(value: AcknowledgeAnalyzerRequest) -> Self {
        Self {
            request_id: value.request_id,
            success: value.success,
        }
    }
}

#[derive(Debug)]
pub struct AcknowledgeAnalyzerResponse {}

impl TryFrom<proto::AcknowledgeAnalyzerResponse> for AcknowledgeAnalyzerResponse {
    type Error = SerDeError;

    fn try_from(_value: proto::AcknowledgeAnalyzerResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<AcknowledgeAnalyzerResponse> for proto::AcknowledgeAnalyzerResponse {
    fn from(_value: AcknowledgeAnalyzerResponse) -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct GetExecuteAnalyzerRequest {}

impl TryFrom<proto::GetExecuteAnalyzerRequest> for GetExecuteAnalyzerRequest {
    type Error = SerDeError;

    fn try_from(_value: proto::GetExecuteAnalyzerRequest) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<GetExecuteAnalyzerRequest> for proto::GetExecuteAnalyzerRequest {
    fn from(_value: GetExecuteAnalyzerRequest) -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct GetExecuteAnalyzerResponse {
    pub execution_job: Option<ExecutionJob>,
    pub request_id: i64,
}

impl TryFrom<proto::GetExecuteAnalyzerResponse> for GetExecuteAnalyzerResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetExecuteAnalyzerResponse) -> Result<Self, Self::Error> {
        let request_id = value.request_id;
        let maybe_job = value.maybe_job.ok_or(Self::Error::MissingField(
            "GetExecuteAnalyzerResponse.execution_job",
        ))?;
        let execution_job: Option<ExecutionJob> = maybe_job.try_into()?;

        let execution_job = execution_job.ok_or(Self::Error::MissingField(
            "GetExecuteAnalyzerResponse.execution_job",
        ))?;

        Ok(Self {
            request_id,
            execution_job: Some(execution_job),
        })
    }
}

impl From<GetExecuteAnalyzerResponse> for proto::GetExecuteAnalyzerResponse {
    fn from(value: GetExecuteAnalyzerResponse) -> Self {
        let execution_job = value.execution_job.into();
        let request_id = value.request_id;
        proto::GetExecuteAnalyzerResponse {
            request_id,
            maybe_job: Some(execution_job),
        }
    }
}

#[derive(Debug)]
pub struct GetExecuteGeneratorRequest {}

impl TryFrom<proto::GetExecuteGeneratorRequest> for GetExecuteGeneratorRequest {
    type Error = SerDeError;

    fn try_from(_value: proto::GetExecuteGeneratorRequest) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<GetExecuteGeneratorRequest> for proto::GetExecuteGeneratorRequest {
    fn from(_value: GetExecuteGeneratorRequest) -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct GetExecuteGeneratorResponse {
    pub execution_job: Option<ExecutionJob>,
    pub request_id: i64,
}

impl TryFrom<proto::GetExecuteGeneratorResponse> for GetExecuteGeneratorResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetExecuteGeneratorResponse) -> Result<Self, Self::Error> {
        let request_id = value.request_id;
        let maybe_job = value.maybe_job.ok_or(Self::Error::MissingField(
            "proto::GetExecuteGeneratorResponse.maybe_job",
        ))?;
        let execution_job: Option<ExecutionJob> = maybe_job.try_into()?;

        let execution_job = execution_job.ok_or(Self::Error::MissingField(
            "proto::GetExecuteGeneratorResponse.execution_job",
        ))?;

        Ok(Self {
            request_id,
            execution_job: Some(execution_job),
        })
    }
}

impl From<GetExecuteGeneratorResponse> for proto::GetExecuteGeneratorResponse {
    fn from(value: GetExecuteGeneratorResponse) -> Self {
        let execution_job = value.execution_job.into();
        let request_id = value.request_id;
        proto::GetExecuteGeneratorResponse {
            request_id,
            maybe_job: Some(execution_job),
        }
    }
}

#[derive(Debug)]
pub struct PutExecuteAnalyzerRequest {
    pub execution_job: ExecutionJob,
}

impl TryFrom<proto::PutExecuteAnalyzerRequest> for PutExecuteAnalyzerRequest {
    type Error = SerDeError;

    fn try_from(value: proto::PutExecuteAnalyzerRequest) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingField(
                "PutExecuteAnalyzerRequest.execution_job",
            ))?
            .try_into()?;

        Ok(Self { execution_job })
    }
}

impl From<PutExecuteAnalyzerRequest> for proto::PutExecuteAnalyzerRequest {
    fn from(value: PutExecuteAnalyzerRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
        }
    }
}

#[derive(Debug)]
pub struct PutExecuteAnalyzerResponse {}

impl TryFrom<proto::PutExecuteAnalyzerResponse> for PutExecuteAnalyzerResponse {
    type Error = SerDeError;

    fn try_from(_value: proto::PutExecuteAnalyzerResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PutExecuteAnalyzerResponse> for proto::PutExecuteAnalyzerResponse {
    fn from(_value: PutExecuteAnalyzerResponse) -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct PutExecuteGeneratorRequest {
    pub execution_job: ExecutionJob,
}

impl TryFrom<proto::PutExecuteGeneratorRequest> for PutExecuteGeneratorRequest {
    type Error = SerDeError;

    fn try_from(value: proto::PutExecuteGeneratorRequest) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingField(
                "PutExecuteGeneratorRequest.execution_job",
            ))?
            .try_into()?;

        Ok(Self { execution_job })
    }
}

impl From<PutExecuteGeneratorRequest> for proto::PutExecuteGeneratorRequest {
    fn from(value: PutExecuteGeneratorRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
        }
    }
}

#[derive(Debug)]
pub struct PutExecuteGeneratorResponse {}

impl TryFrom<proto::PutExecuteGeneratorResponse> for PutExecuteGeneratorResponse {
    type Error = SerDeError;

    fn try_from(_value: proto::PutExecuteGeneratorResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PutExecuteGeneratorResponse> for proto::PutExecuteGeneratorResponse {
    fn from(_value: PutExecuteGeneratorResponse) -> Self {
        Self {}
    }
}

impl From<Option<ExecutionJob>> for get_execute_generator_response::MaybeJob {
    fn from(execution_job: Option<ExecutionJob>) -> Self {
        match execution_job {
            None => get_execute_generator_response::MaybeJob::NoJobs(proto::NoAvailableJobs{}),
            Some(job) => get_execute_generator_response::MaybeJob::Job(job.into()),
        }
    }
}

impl TryFrom<get_execute_analyzer_response::MaybeJob> for Option<ExecutionJob> {
    type Error = SerDeError;

    fn try_from(maybe_job: get_execute_analyzer_response::MaybeJob) -> Result<Self, Self::Error> {
        let maybe_job = match maybe_job {
            get_execute_analyzer_response::MaybeJob::Job(job) => Some(job.try_into()?),
            get_execute_analyzer_response::MaybeJob::NoJobs(_) => None,
        };
        Ok(maybe_job)
    }
}

impl From<Option<ExecutionJob>> for get_execute_analyzer_response::MaybeJob {
    fn from(execution_job: Option<ExecutionJob>) -> Self {
        match execution_job {
            None => get_execute_analyzer_response::MaybeJob::NoJobs(proto::NoAvailableJobs{}),
            Some(job) => get_execute_analyzer_response::MaybeJob::Job(job.into()),
        }
    }
}

impl TryFrom<get_execute_generator_response::MaybeJob> for Option<ExecutionJob> {
    type Error = SerDeError;

    fn try_from(maybe_job: get_execute_generator_response::MaybeJob) -> Result<Self, Self::Error> {
        let maybe_job = match maybe_job {
            get_execute_generator_response::MaybeJob::Job(job) => Some(job.try_into()?),
            get_execute_generator_response::MaybeJob::NoJobs(_) => None,
        };
        Ok(maybe_job)
    }
}

