#![allow(unused_variables)]
pub use crate::graplinc::grapl::api::plugin_work_queue::v1beta1::{
    plugin_work_queue_service_client,
    plugin_work_queue_service_server,
    AcknowledgeGeneratorRequest as AcknowledgeGeneratorRequestProto,
    AcknowledgeGeneratorResponse as AcknowledgeGeneratorResponseProto,
    AcknowledgeAnalyzerRequest as AcknowledgeAnalyzerRequestProto,
    AcknowledgeAnalyzerResponse as AcknowledgeAnalyzerResponseProto,
    ExecutionJob as ExecutionJobProto,
    get_execute_analyzer_response,
    get_execute_generator_response,
    GetExecuteAnalyzerRequest as GetExecuteAnalyzerRequestProto,
    GetExecuteAnalyzerResponse as GetExecuteAnalyzerResponseProto,
    GetExecuteGeneratorRequest as GetExecuteGeneratorRequestProto,
    GetExecuteGeneratorResponse as GetExecuteGeneratorResponseProto,
    PutExecuteAnalyzerRequest as PutExecuteAnalyzerRequestProto,
    PutExecuteAnalyzerResponse as PutExecuteAnalyzerResponseProto,
    PutExecuteGeneratorRequest as PutExecuteGeneratorRequestProto,
    PutExecuteGeneratorResponse as PutExecuteGeneratorResponseProto,
};

#[derive(Debug, thiserror::Error)]
pub enum PluginWorkQueueDeserializationError {
    #[error("Missing a required field")]
    MissingRequiredField(&'static str),
    #[error("Empty field")]
    EmptyField(&'static str),
}

pub struct ExecutionJob {
    pub tenant_id: uuid::Uuid,
    pub plugin_id: uuid::Uuid,
    pub data: Vec<u8>,
}

impl TryFrom<ExecutionJobProto> for ExecutionJob {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: ExecutionJobProto) -> Result<Self, Self::Error> {
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

impl From<ExecutionJob> for ExecutionJobProto {
    fn from(value: ExecutionJob) -> Self {
        assert!(!value.data.is_empty());

        Self {
            tenant_id: Some(value.tenant_id.into()),
            plugin_id: Some(value.plugin_id.into()),
            data: value.data,
        }
    }
}

pub struct AcknowledgeGeneratorRequest {
    pub request_id: i64,
    pub success: bool,
}

impl TryFrom<AcknowledgeGeneratorRequestProto> for AcknowledgeGeneratorRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: AcknowledgeGeneratorRequestProto) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id;
        let success = value.success;
        Ok(Self { request_id, success })
    }
}

impl From<AcknowledgeGeneratorRequest> for AcknowledgeGeneratorRequestProto {
    fn from(value: AcknowledgeGeneratorRequest) -> Self {
        Self {
            request_id: value.request_id,
            success: value.success
        }
    }
}

pub struct AcknowledgeGeneratorResponse {}

impl TryFrom<AcknowledgeGeneratorResponseProto> for AcknowledgeGeneratorResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: AcknowledgeGeneratorResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<AcknowledgeGeneratorResponse> for AcknowledgeGeneratorResponseProto {
    fn from(_value: AcknowledgeGeneratorResponse) -> Self {
        Self {}
    }
}

pub struct AcknowledgeAnalyzerRequest {
    pub request_id: i64,
    pub success: bool,
}

impl TryFrom<AcknowledgeAnalyzerRequestProto> for AcknowledgeAnalyzerRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: AcknowledgeAnalyzerRequestProto) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id;

        let success = value.success;
        Ok(Self { request_id, success })
    }
}

impl From<AcknowledgeAnalyzerRequest> for AcknowledgeAnalyzerRequestProto {
    fn from(value: AcknowledgeAnalyzerRequest) -> Self {
        Self {
            request_id: value.request_id,
            success: value.success,
        }
    }
}

pub struct AcknowledgeAnalyzerResponse {}

impl TryFrom<AcknowledgeAnalyzerResponseProto> for AcknowledgeAnalyzerResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: AcknowledgeAnalyzerResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<AcknowledgeAnalyzerResponse> for AcknowledgeAnalyzerResponseProto {
    fn from(_value: AcknowledgeAnalyzerResponse) -> Self {
        Self {}
    }
}


pub struct GetExecuteAnalyzerRequest {}

impl TryFrom<GetExecuteAnalyzerRequestProto> for GetExecuteAnalyzerRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: GetExecuteAnalyzerRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<GetExecuteAnalyzerRequest> for GetExecuteAnalyzerRequestProto {
    fn from(_value: GetExecuteAnalyzerRequest) -> Self {
        Self {}
    }
}

pub struct GetExecuteAnalyzerResponse {
    pub execution_job: Option<ExecutionJob>,
    pub request_id: i64,
}

impl TryFrom<GetExecuteAnalyzerResponseProto> for GetExecuteAnalyzerResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: GetExecuteAnalyzerResponseProto) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id;
        let maybe_job = value.maybe_job.ok_or(Self::Error::MissingRequiredField(
                "GetExecuteAnalyzerResponse.execution_job",
            ))?;
        let execution_job = match maybe_job {
            get_execute_analyzer_response::MaybeJob::Job(job) => {Some(job)}
            get_execute_analyzer_response::MaybeJob::NoJobs(_) => {None}
        };

        let execution_job = execution_job.ok_or(Self::Error::MissingRequiredField(
            "GetExecuteAnalyzerResponse.execution_job",
        ))?.try_into()?;

        Ok(Self {
            request_id,
            execution_job: Some(execution_job),
        })

    }
}

impl From<GetExecuteAnalyzerResponse> for GetExecuteAnalyzerResponseProto {
    fn from(value: GetExecuteAnalyzerResponse) -> Self {
        todo!()
        // Self {
        //     execution_job: Some(value.execution_job.into()),
        //     request_id: value.request_id,
        // }
    }
}

pub struct GetExecuteGeneratorRequest {}

impl TryFrom<GetExecuteGeneratorRequestProto> for GetExecuteGeneratorRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: GetExecuteGeneratorRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<GetExecuteGeneratorRequest> for GetExecuteGeneratorRequestProto {
    fn from(_value: GetExecuteGeneratorRequest) -> Self {
        Self {}
    }
}

pub struct GetExecuteGeneratorResponse {
    pub execution_job: Option<ExecutionJob>,
    pub request_id: i64,
}

impl TryFrom<GetExecuteGeneratorResponseProto> for GetExecuteGeneratorResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: GetExecuteGeneratorResponseProto) -> Result<Self, Self::Error> {
        let request_id = value
            .request_id;
        let maybe_job = value.maybe_job.ok_or(Self::Error::MissingRequiredField(
            "GetExecuteGeneratorResponseProto.maybe_job",
        ))?;
        let execution_job = match maybe_job {
            get_execute_generator_response::MaybeJob::Job(job) => {Some(job)}
            get_execute_generator_response::MaybeJob::NoJobs(_) => {None}
        };

        let execution_job = execution_job.ok_or(Self::Error::MissingRequiredField(
            "GetExecuteGeneratorResponseProto.execution_job",
        ))?.try_into()?;

        Ok(Self {
            request_id,
            execution_job: Some(execution_job),
        })
    }
}

impl From<GetExecuteGeneratorResponse> for GetExecuteGeneratorResponseProto {
    fn from(value: GetExecuteGeneratorResponse) -> Self {
        // Self {
        //     execution_job: Some(value.execution_job.into()),
        //     request_id: value.request_id,
        // }
        todo!()
    }
}

pub struct PutExecuteAnalyzerRequest {
    pub execution_job: ExecutionJob,
    pub trace_id: uuid::Uuid,
}

impl TryFrom<PutExecuteAnalyzerRequestProto> for PutExecuteAnalyzerRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: PutExecuteAnalyzerRequestProto) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "PutExecuteAnalyzerRequest.execution_job",
            ))?
            .try_into()?;

        let trace_id = value
            .trace_id
            .ok_or(Self::Error::MissingRequiredField(
                "PutExecuteAnalyzerRequest.trace_id",
            ))?
            .into();
        Ok(Self { execution_job, trace_id })
    }
}

impl From<PutExecuteAnalyzerRequest> for PutExecuteAnalyzerRequestProto {
    fn from(value: PutExecuteAnalyzerRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
            trace_id: Some(value.trace_id.into()),
        }
    }
}

pub struct PutExecuteAnalyzerResponse {}

impl TryFrom<PutExecuteAnalyzerResponseProto> for PutExecuteAnalyzerResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: PutExecuteAnalyzerResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PutExecuteAnalyzerResponse> for PutExecuteAnalyzerResponseProto {
    fn from(_value: PutExecuteAnalyzerResponse) -> Self {
        Self {}
    }
}

pub struct PutExecuteGeneratorRequest {
    pub execution_job: ExecutionJob,
    pub trace_id: uuid::Uuid,
}

impl TryFrom<PutExecuteGeneratorRequestProto> for PutExecuteGeneratorRequest {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(value: PutExecuteGeneratorRequestProto) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingRequiredField(
                "PutExecuteGeneratorRequest.execution_job",
            ))?
            .try_into()?;


        let trace_id = value
            .trace_id
            .ok_or(Self::Error::MissingRequiredField(
                "PutExecuteAnalyzerRequest.trace_id",
            ))?
            .into();
        Ok(Self { execution_job, trace_id })
    }
}

impl From<PutExecuteGeneratorRequest> for PutExecuteGeneratorRequestProto {
    fn from(value: PutExecuteGeneratorRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
            trace_id: Some(value.trace_id.into()),
        }
    }
}

pub struct PutExecuteGeneratorResponse {}

impl TryFrom<PutExecuteGeneratorResponseProto> for PutExecuteGeneratorResponse {
    type Error = PluginWorkQueueDeserializationError;

    fn try_from(_value: PutExecuteGeneratorResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PutExecuteGeneratorResponse> for PutExecuteGeneratorResponseProto {
    fn from(_value: PutExecuteGeneratorResponse) -> Self {
        Self {}
    }
}
