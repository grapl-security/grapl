#![allow(unused_variables)]

use std::fmt::Formatter;

use bytes::Bytes;
use proto::{
    get_execute_analyzer_response,
    get_execute_generator_response,
};
use uuid::Uuid;

pub use crate::graplinc::grapl::api::plugin_work_queue::{
    v1beta1_client::PluginWorkQueueClient,
    v1beta1_server::{
        PluginWorkQueueApi,
        PluginWorkQueueServer,
    },
};
use crate::{
    graplinc::grapl::api::{
        graph::v1beta1::GraphDescription,
        plugin_sdk::analyzers::v1beta1::messages::ExecutionResult,
    },
    protobufs::graplinc::grapl::api::plugin_work_queue::v1beta1 as proto,
    serde_impl::ProtobufSerializable,
    type_url,
    SerDeError,
};

#[derive(Clone, PartialEq, Eq)]
pub struct ExecutionJob {
    data: Bytes,
    tenant_id: Uuid,
    trace_id: Uuid,
    event_source_id: Uuid,
}

impl ExecutionJob {
    pub fn new(data: Bytes, tenant_id: Uuid, trace_id: Uuid, event_source_id: Uuid) -> Self {
        Self {
            data,
            tenant_id,
            trace_id,
            event_source_id,
        }
    }

    pub fn data(self) -> Bytes {
        self.data
    }

    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    pub fn trace_id(&self) -> Uuid {
        self.trace_id
    }

    pub fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }
}

impl std::fmt::Debug for ExecutionJob {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionJob")
            .field("data.len", &self.data.len())
            .finish()
    }
}

impl TryFrom<proto::ExecutionJob> for ExecutionJob {
    type Error = SerDeError;

    fn try_from(value: proto::ExecutionJob) -> Result<Self, Self::Error> {
        let tenant_id = value
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        let trace_id = value.trace_id.ok_or(SerDeError::MissingField("trace_id"))?;

        let event_source_id = value
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?;

        let data = value.data;

        Ok(Self {
            data,
            tenant_id: tenant_id.into(),
            trace_id: trace_id.into(),
            event_source_id: event_source_id.into(),
        })
    }
}

impl From<ExecutionJob> for proto::ExecutionJob {
    fn from(value: ExecutionJob) -> Self {
        Self {
            data: value.data,
            tenant_id: Some(value.tenant_id.into()),
            trace_id: Some(value.trace_id.into()),
            event_source_id: Some(value.event_source_id.into()),
        }
    }
}

impl ProtobufSerializable for ExecutionJob {
    type ProtobufMessage = proto::ExecutionJob;
}

impl type_url::TypeUrl for ExecutionJob {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.ExecutionJob";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcknowledgeGeneratorRequest {
    request_id: i64,
    graph_description: Option<GraphDescription>,
    plugin_id: Uuid,
    tenant_id: Uuid,
    trace_id: Uuid,
    event_source_id: Uuid,
}

impl AcknowledgeGeneratorRequest {
    pub fn new(
        request_id: i64,
        graph_description: Option<GraphDescription>,
        plugin_id: Uuid,
        tenant_id: Uuid,
        trace_id: Uuid,
        event_source_id: Uuid,
    ) -> Self {
        Self {
            request_id,
            graph_description,
            plugin_id,
            tenant_id,
            trace_id,
            event_source_id,
        }
    }

    pub fn request_id(&self) -> i64 {
        self.request_id
    }

    pub fn graph_description(self) -> Option<GraphDescription> {
        self.graph_description
    }

    pub fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }

    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    pub fn trace_id(&self) -> Uuid {
        self.trace_id
    }

    pub fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }
}

impl TryFrom<proto::AcknowledgeGeneratorRequest> for AcknowledgeGeneratorRequest {
    type Error = SerDeError;

    fn try_from(value: proto::AcknowledgeGeneratorRequest) -> Result<Self, Self::Error> {
        let request_id = value.request_id;
        let graph_description = value.graph_description.map(TryInto::try_into).transpose()?;
        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingField("plugin_id"))?
            .into();
        let tenant_id = value
            .tenant_id
            .ok_or(Self::Error::MissingField("tenant_id"))?
            .into();
        let trace_id = value
            .trace_id
            .ok_or(Self::Error::MissingField("trace_id"))?
            .into();
        let event_source_id = value
            .event_source_id
            .ok_or(Self::Error::MissingField("event_source_id"))?
            .into();

        Ok(Self {
            request_id,
            graph_description,
            plugin_id,
            tenant_id,
            trace_id,
            event_source_id,
        })
    }
}

impl From<AcknowledgeGeneratorRequest> for proto::AcknowledgeGeneratorRequest {
    fn from(value: AcknowledgeGeneratorRequest) -> Self {
        Self {
            request_id: value.request_id,
            graph_description: value.graph_description.map(Into::into),
            plugin_id: Some(value.plugin_id.into()),
            tenant_id: Some(value.tenant_id.into()),
            trace_id: Some(value.trace_id.into()),
            event_source_id: Some(value.event_source_id.into()),
        }
    }
}

impl ProtobufSerializable for AcknowledgeGeneratorRequest {
    type ProtobufMessage = proto::AcknowledgeGeneratorRequest;
}

impl type_url::TypeUrl for AcknowledgeGeneratorRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.AcknowledgeGeneratorRequest";
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ProtobufSerializable for AcknowledgeGeneratorResponse {
    type ProtobufMessage = proto::AcknowledgeGeneratorResponse;
}

impl type_url::TypeUrl for AcknowledgeGeneratorResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.AcknowledgeGeneratorResponse";
}

#[derive(Debug, Clone, PartialEq)]
pub struct AcknowledgeAnalyzerRequest {
    request_id: i64,
    execution_result: Option<ExecutionResult>,
    plugin_id: Uuid,
    tenant_id: Uuid,
    trace_id: Uuid,
    event_source_id: Uuid,
}

impl AcknowledgeAnalyzerRequest {
    pub fn new(
        request_id: i64,
        execution_result: Option<ExecutionResult>,
        plugin_id: Uuid,
        tenant_id: Uuid,
        trace_id: Uuid,
        event_source_id: Uuid,
    ) -> Self {
        Self {
            request_id,
            execution_result,
            plugin_id,
            tenant_id,
            trace_id,
            event_source_id,
        }
    }

    pub fn request_id(&self) -> i64 {
        self.request_id
    }

    pub fn execution_result(self) -> Option<ExecutionResult> {
        self.execution_result
    }

    pub fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }

    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    pub fn trace_id(&self) -> Uuid {
        self.trace_id
    }

    pub fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }
}

impl TryFrom<proto::AcknowledgeAnalyzerRequest> for AcknowledgeAnalyzerRequest {
    type Error = SerDeError;

    fn try_from(value: proto::AcknowledgeAnalyzerRequest) -> Result<Self, Self::Error> {
        let request_id = value.request_id;
        let execution_result = value.execution_result.map(TryInto::try_into).transpose()?;
        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingField("plugin_id"))?
            .into();
        let tenant_id = value
            .tenant_id
            .ok_or(Self::Error::MissingField("tenant_id"))?
            .into();
        let trace_id = value
            .trace_id
            .ok_or(Self::Error::MissingField("trace_id"))?
            .into();
        let event_source_id = value
            .event_source_id
            .ok_or(Self::Error::MissingField("event_source_id"))?
            .into();

        Ok(Self {
            request_id,
            execution_result,
            plugin_id,
            tenant_id,
            trace_id,
            event_source_id,
        })
    }
}

impl From<AcknowledgeAnalyzerRequest> for proto::AcknowledgeAnalyzerRequest {
    fn from(value: AcknowledgeAnalyzerRequest) -> Self {
        Self {
            request_id: value.request_id,
            execution_result: value.execution_result.map(Into::into),
            plugin_id: Some(value.plugin_id.into()),
            tenant_id: Some(value.tenant_id.into()),
            trace_id: Some(value.trace_id.into()),
            event_source_id: Some(value.event_source_id.into()),
        }
    }
}

impl ProtobufSerializable for AcknowledgeAnalyzerRequest {
    type ProtobufMessage = proto::AcknowledgeAnalyzerRequest;
}

impl type_url::TypeUrl for AcknowledgeAnalyzerRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.AcknowledgeAnalyzerRequest";
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ProtobufSerializable for AcknowledgeAnalyzerResponse {
    type ProtobufMessage = proto::AcknowledgeAnalyzerResponse;
}

impl type_url::TypeUrl for AcknowledgeAnalyzerResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.AcknowledgeAnalyzerResponse";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetExecuteAnalyzerRequest {
    plugin_id: Uuid,
}

impl GetExecuteAnalyzerRequest {
    pub fn new(plugin_id: Uuid) -> Self {
        Self { plugin_id }
    }

    pub fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }
}

impl TryFrom<proto::GetExecuteAnalyzerRequest> for GetExecuteAnalyzerRequest {
    type Error = SerDeError;

    fn try_from(value: proto::GetExecuteAnalyzerRequest) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingField("plugin_id"))?
            .into();
        Ok(Self { plugin_id })
    }
}

impl From<GetExecuteAnalyzerRequest> for proto::GetExecuteAnalyzerRequest {
    fn from(value: GetExecuteAnalyzerRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

impl ProtobufSerializable for GetExecuteAnalyzerRequest {
    type ProtobufMessage = proto::GetExecuteAnalyzerRequest;
}

impl type_url::TypeUrl for GetExecuteAnalyzerRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.GetExecuteAnalyzerRequest";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetExecuteAnalyzerResponse {
    execution_job: Option<ExecutionJob>,
    request_id: i64,
}

impl GetExecuteAnalyzerResponse {
    pub fn new(execution_job: Option<ExecutionJob>, request_id: i64) -> Self {
        Self {
            execution_job,
            request_id,
        }
    }

    pub fn execution_job(self) -> Option<ExecutionJob> {
        self.execution_job
    }

    pub fn request_id(&self) -> i64 {
        self.request_id
    }
}

impl TryFrom<proto::GetExecuteAnalyzerResponse> for GetExecuteAnalyzerResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetExecuteAnalyzerResponse) -> Result<Self, Self::Error> {
        let request_id = value.request_id;
        let maybe_job = value.maybe_job.ok_or(Self::Error::MissingField(
            "GetExecuteAnalyzerResponse.maybe_job",
        ))?;
        let execution_job: Option<ExecutionJob> = maybe_job.try_into()?;

        Ok(Self {
            request_id,
            execution_job,
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

impl ProtobufSerializable for GetExecuteAnalyzerResponse {
    type ProtobufMessage = proto::GetExecuteAnalyzerResponse;
}

impl type_url::TypeUrl for GetExecuteAnalyzerResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.GetExecuteAnalyzerResponse";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetExecuteGeneratorRequest {
    plugin_id: Uuid,
}

impl GetExecuteGeneratorRequest {
    pub fn new(plugin_id: Uuid) -> Self {
        Self { plugin_id }
    }

    pub fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }
}

impl TryFrom<proto::GetExecuteGeneratorRequest> for GetExecuteGeneratorRequest {
    type Error = SerDeError;

    fn try_from(value: proto::GetExecuteGeneratorRequest) -> Result<Self, Self::Error> {
        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingField("plugin_id"))?
            .into();
        Ok(Self { plugin_id })
    }
}

impl From<GetExecuteGeneratorRequest> for proto::GetExecuteGeneratorRequest {
    fn from(value: GetExecuteGeneratorRequest) -> Self {
        Self {
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

impl ProtobufSerializable for GetExecuteGeneratorRequest {
    type ProtobufMessage = proto::GetExecuteGeneratorRequest;
}

impl type_url::TypeUrl for GetExecuteGeneratorRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.GetExecuteGeneratorRequest";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetExecuteGeneratorResponse {
    execution_job: Option<ExecutionJob>,
    request_id: i64,
}

impl GetExecuteGeneratorResponse {
    pub fn new(execution_job: Option<ExecutionJob>, request_id: i64) -> Self {
        Self {
            execution_job,
            request_id,
        }
    }

    pub fn execution_job(self) -> Option<ExecutionJob> {
        self.execution_job
    }

    pub fn request_id(&self) -> i64 {
        self.request_id
    }
}

impl TryFrom<proto::GetExecuteGeneratorResponse> for GetExecuteGeneratorResponse {
    type Error = SerDeError;

    fn try_from(value: proto::GetExecuteGeneratorResponse) -> Result<Self, Self::Error> {
        let request_id = value.request_id;
        let maybe_job = value.maybe_job.ok_or(Self::Error::MissingField(
            "proto::GetExecuteGeneratorResponse.maybe_job",
        ))?;
        let execution_job: Option<ExecutionJob> = maybe_job.try_into()?;

        Ok(Self {
            request_id,
            execution_job,
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

impl ProtobufSerializable for GetExecuteGeneratorResponse {
    type ProtobufMessage = proto::GetExecuteGeneratorResponse;
}

impl type_url::TypeUrl for GetExecuteGeneratorResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.GetExecuteGeneratorResponse";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushExecuteAnalyzerRequest {
    execution_job: ExecutionJob,
    plugin_id: Uuid,
}

impl PushExecuteAnalyzerRequest {
    pub fn new(execution_job: ExecutionJob, plugin_id: Uuid) -> Self {
        Self {
            execution_job,
            plugin_id,
        }
    }

    pub fn execution_job(self) -> ExecutionJob {
        self.execution_job
    }

    pub fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }
}

impl TryFrom<proto::PushExecuteAnalyzerRequest> for PushExecuteAnalyzerRequest {
    type Error = SerDeError;

    fn try_from(value: proto::PushExecuteAnalyzerRequest) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingField("execution_job"))?
            .try_into()?;

        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingField("plugin_id"))?
            .into();

        Ok(Self {
            execution_job,
            plugin_id,
        })
    }
}

impl From<PushExecuteAnalyzerRequest> for proto::PushExecuteAnalyzerRequest {
    fn from(value: PushExecuteAnalyzerRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

impl ProtobufSerializable for PushExecuteAnalyzerRequest {
    type ProtobufMessage = proto::PushExecuteAnalyzerRequest;
}

impl type_url::TypeUrl for PushExecuteAnalyzerRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.PushExecuteAnalyzerRequest";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushExecuteAnalyzerResponse {}

impl TryFrom<proto::PushExecuteAnalyzerResponse> for PushExecuteAnalyzerResponse {
    type Error = SerDeError;

    fn try_from(_value: proto::PushExecuteAnalyzerResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PushExecuteAnalyzerResponse> for proto::PushExecuteAnalyzerResponse {
    fn from(_value: PushExecuteAnalyzerResponse) -> Self {
        Self {}
    }
}

impl ProtobufSerializable for PushExecuteAnalyzerResponse {
    type ProtobufMessage = proto::PushExecuteAnalyzerResponse;
}

impl type_url::TypeUrl for PushExecuteAnalyzerResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.PushExecuteAnalyzerResponse";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushExecuteGeneratorRequest {
    execution_job: ExecutionJob,
    plugin_id: Uuid,
}

impl PushExecuteGeneratorRequest {
    pub fn new(execution_job: ExecutionJob, plugin_id: Uuid) -> Self {
        Self {
            execution_job,
            plugin_id,
        }
    }

    pub fn execution_job(self) -> ExecutionJob {
        self.execution_job
    }

    pub fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }
}

impl TryFrom<proto::PushExecuteGeneratorRequest> for PushExecuteGeneratorRequest {
    type Error = SerDeError;

    fn try_from(value: proto::PushExecuteGeneratorRequest) -> Result<Self, Self::Error> {
        let execution_job = value
            .execution_job
            .ok_or(Self::Error::MissingField("execution_job"))?
            .try_into()?;

        let plugin_id = value
            .plugin_id
            .ok_or(Self::Error::MissingField("plugin_id"))?
            .into();

        Ok(Self {
            execution_job,
            plugin_id,
        })
    }
}

impl From<PushExecuteGeneratorRequest> for proto::PushExecuteGeneratorRequest {
    fn from(value: PushExecuteGeneratorRequest) -> Self {
        Self {
            execution_job: Some(value.execution_job.into()),
            plugin_id: Some(value.plugin_id.into()),
        }
    }
}

impl ProtobufSerializable for PushExecuteGeneratorRequest {
    type ProtobufMessage = proto::PushExecuteGeneratorRequest;
}

impl type_url::TypeUrl for PushExecuteGeneratorRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.PushExecuteGeneratorRequest";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushExecuteGeneratorResponse {}

impl TryFrom<proto::PushExecuteGeneratorResponse> for PushExecuteGeneratorResponse {
    type Error = SerDeError;

    fn try_from(_value: proto::PushExecuteGeneratorResponse) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<PushExecuteGeneratorResponse> for proto::PushExecuteGeneratorResponse {
    fn from(_value: PushExecuteGeneratorResponse) -> Self {
        Self {}
    }
}

impl ProtobufSerializable for PushExecuteGeneratorResponse {
    type ProtobufMessage = proto::PushExecuteGeneratorResponse;
}

impl type_url::TypeUrl for PushExecuteGeneratorResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.PushExecuteGeneratorResponse";
}

impl From<Option<ExecutionJob>> for get_execute_generator_response::MaybeJob {
    fn from(execution_job: Option<ExecutionJob>) -> Self {
        match execution_job {
            None => get_execute_generator_response::MaybeJob::NoJobs(proto::NoAvailableJobs {}),
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
            None => get_execute_analyzer_response::MaybeJob::NoJobs(proto::NoAvailableJobs {}),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueDepthForAnalyzerRequest {
    analyzer_id: Uuid,
}

impl QueueDepthForAnalyzerRequest {
    pub fn new(analyzer_id: Uuid) -> Self {
        Self { analyzer_id }
    }

    pub fn analyzer_id(&self) -> Uuid {
        self.analyzer_id
    }
}

impl TryFrom<proto::QueueDepthForAnalyzerRequest> for QueueDepthForAnalyzerRequest {
    type Error = SerDeError;

    fn try_from(request: proto::QueueDepthForAnalyzerRequest) -> Result<Self, Self::Error> {
        let analyzer_id = request
            .analyzer_id
            .ok_or(SerDeError::MissingField("analyzer_id"))?
            .into();

        Ok(Self { analyzer_id })
    }
}

impl From<QueueDepthForAnalyzerRequest> for proto::QueueDepthForAnalyzerRequest {
    fn from(request: QueueDepthForAnalyzerRequest) -> Self {
        Self {
            analyzer_id: Some(request.analyzer_id().into()),
        }
    }
}

impl ProtobufSerializable for QueueDepthForAnalyzerRequest {
    type ProtobufMessage = proto::QueueDepthForAnalyzerRequest;
}

impl type_url::TypeUrl for QueueDepthForAnalyzerRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.QueueDepthForAnalyzerRequest";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueDepthForAnalyzerResponse {
    queue_depth: u32,
    dominant_event_source_id: Uuid,
}

impl QueueDepthForAnalyzerResponse {
    pub fn new(queue_depth: u32, dominant_event_source_id: Uuid) -> Self {
        Self {
            queue_depth,
            dominant_event_source_id,
        }
    }

    pub fn queue_depth(&self) -> u32 {
        self.queue_depth
    }

    pub fn dominant_event_source_id(&self) -> Uuid {
        self.dominant_event_source_id
    }
}

impl TryFrom<proto::QueueDepthForAnalyzerResponse> for QueueDepthForAnalyzerResponse {
    type Error = SerDeError;

    fn try_from(response: proto::QueueDepthForAnalyzerResponse) -> Result<Self, Self::Error> {
        let dominant_event_source_id = response
            .dominant_event_source_id
            .ok_or(SerDeError::MissingField("dominant_event_source_id"))?
            .into();

        Ok(Self {
            queue_depth: response.queue_depth,
            dominant_event_source_id,
        })
    }
}

impl From<QueueDepthForAnalyzerResponse> for proto::QueueDepthForAnalyzerResponse {
    fn from(response: QueueDepthForAnalyzerResponse) -> Self {
        Self {
            queue_depth: response.queue_depth(),
            dominant_event_source_id: Some(response.dominant_event_source_id().into()),
        }
    }
}

impl ProtobufSerializable for QueueDepthForAnalyzerResponse {
    type ProtobufMessage = proto::QueueDepthForAnalyzerResponse;
}

impl type_url::TypeUrl for QueueDepthForAnalyzerResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.QueueDepthForAnalyzerResponse";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueDepthForGeneratorRequest {
    generator_id: Uuid,
}

impl QueueDepthForGeneratorRequest {
    pub fn new(generator_id: Uuid) -> Self {
        Self { generator_id }
    }

    pub fn generator_id(&self) -> Uuid {
        self.generator_id
    }
}

impl TryFrom<proto::QueueDepthForGeneratorRequest> for QueueDepthForGeneratorRequest {
    type Error = SerDeError;

    fn try_from(request: proto::QueueDepthForGeneratorRequest) -> Result<Self, Self::Error> {
        let generator_id = request
            .generator_id
            .ok_or(SerDeError::MissingField("generator_id"))?
            .into();

        Ok(Self { generator_id })
    }
}

impl From<QueueDepthForGeneratorRequest> for proto::QueueDepthForGeneratorRequest {
    fn from(request: QueueDepthForGeneratorRequest) -> Self {
        Self {
            generator_id: Some(request.generator_id().into()),
        }
    }
}

impl ProtobufSerializable for QueueDepthForGeneratorRequest {
    type ProtobufMessage = proto::QueueDepthForGeneratorRequest;
}

impl type_url::TypeUrl for QueueDepthForGeneratorRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.QueueDepthForGeneratorRequest";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueDepthForGeneratorResponse {
    queue_depth: u32,
    event_source_id: Uuid,
}

impl QueueDepthForGeneratorResponse {
    pub fn new(queue_depth: u32, event_source_id: Uuid) -> Self {
        Self {
            queue_depth,
            event_source_id,
        }
    }

    pub fn queue_depth(&self) -> u32 {
        self.queue_depth
    }

    pub fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }
}

impl TryFrom<proto::QueueDepthForGeneratorResponse> for QueueDepthForGeneratorResponse {
    type Error = SerDeError;

    fn try_from(response: proto::QueueDepthForGeneratorResponse) -> Result<Self, Self::Error> {
        let event_source_id = response
            .event_source_id
            .ok_or(SerDeError::MissingField("event_source_id"))?
            .into();

        Ok(Self {
            queue_depth: response.queue_depth,
            event_source_id,
        })
    }
}

impl From<QueueDepthForGeneratorResponse> for proto::QueueDepthForGeneratorResponse {
    fn from(response: QueueDepthForGeneratorResponse) -> Self {
        Self {
            queue_depth: response.queue_depth,
            event_source_id: Some(response.event_source_id().into()),
        }
    }
}

impl ProtobufSerializable for QueueDepthForGeneratorResponse {
    type ProtobufMessage = proto::QueueDepthForGeneratorResponse;
}

impl type_url::TypeUrl for QueueDepthForGeneratorResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_work_queue.v1beta1.QueueDepthForGeneratorResponse";
}
