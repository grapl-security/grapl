use std::time::SystemTime;

use crate::{
    graplinc::grapl::{
        api::graph_query_service::v1beta1::messages::GraphView,
        common::v1beta1::types::{
            EdgeName,
            PropertyName,
            Uid,
        },
    },
    protobufs::graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::{
        execution_result::Inner as ExecutionResultInnerProto,
        update::Inner as UpdateInnerProto,
        AnalyzerName as AnalyzerNameProto,
        EdgeUpdate as EdgeUpdateProto,
        ExecutionHit as ExecutionHitProto,
        ExecutionMiss as ExecutionMissProto,
        ExecutionResult as ExecutionResultProto,
        Int64PropertyUpdate as Int64PropertyUpdateProto,
        LensRef as LensRefProto,
        RunAnalyzerRequest as RunAnalyzerRequestProto,
        RunAnalyzerResponse as RunAnalyzerResponseProto,
        StringPropertyUpdate as StringPropertyUpdateProto,
        UInt64PropertyUpdate as UInt64PropertyUpdateProto,
        Update as UpdateProto,
        Updates as UpdatesProto,
    },
    serde_impl::ProtobufSerializable,
    type_url,
    SerDeError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringPropertyUpdate {
    pub uid: Uid,
    pub property_name: PropertyName,
}

impl TryFrom<StringPropertyUpdateProto> for StringPropertyUpdate {
    type Error = SerDeError;
    fn try_from(value: StringPropertyUpdateProto) -> Result<Self, Self::Error> {
        Ok(Self {
            uid: value
                .uid
                .ok_or(SerDeError::MissingField("uid"))?
                .try_into()?,
            property_name: value
                .property_name
                .ok_or(SerDeError::MissingField("property_name"))?
                .try_into()?,
        })
    }
}

impl From<StringPropertyUpdate> for StringPropertyUpdateProto {
    fn from(value: StringPropertyUpdate) -> Self {
        Self {
            uid: Some(value.uid.into()),
            property_name: Some(value.property_name.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UInt64PropertyUpdate {
    pub uid: Uid,
    pub property_name: PropertyName,
}

impl TryFrom<UInt64PropertyUpdateProto> for UInt64PropertyUpdate {
    type Error = SerDeError;
    fn try_from(value: UInt64PropertyUpdateProto) -> Result<Self, Self::Error> {
        Ok(Self {
            uid: value
                .uid
                .ok_or(SerDeError::MissingField("uid"))?
                .try_into()?,
            property_name: value
                .property_name
                .ok_or(SerDeError::MissingField("property_name"))?
                .try_into()?,
        })
    }
}

impl From<UInt64PropertyUpdate> for UInt64PropertyUpdateProto {
    fn from(value: UInt64PropertyUpdate) -> Self {
        Self {
            uid: Some(value.uid.into()),
            property_name: Some(value.property_name.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Int64PropertyUpdate {
    pub uid: Uid,
    pub property_name: PropertyName,
}

impl TryFrom<Int64PropertyUpdateProto> for Int64PropertyUpdate {
    type Error = SerDeError;
    fn try_from(value: Int64PropertyUpdateProto) -> Result<Self, Self::Error> {
        Ok(Self {
            uid: value
                .uid
                .ok_or(SerDeError::MissingField("uid"))?
                .try_into()?,
            property_name: value
                .property_name
                .ok_or(SerDeError::MissingField("property_name"))?
                .try_into()?,
        })
    }
}

impl From<Int64PropertyUpdate> for Int64PropertyUpdateProto {
    fn from(value: Int64PropertyUpdate) -> Self {
        Self {
            uid: Some(value.uid.into()),
            property_name: Some(value.property_name.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeUpdate {
    pub src_uid: Uid,
    pub dst_uid: Uid,
    pub forward_edge_name: EdgeName,
    pub reverse_edge_name: EdgeName,
}

impl TryFrom<EdgeUpdateProto> for EdgeUpdate {
    type Error = SerDeError;
    fn try_from(value: EdgeUpdateProto) -> Result<Self, Self::Error> {
        Ok(Self {
            src_uid: value
                .src_uid
                .ok_or(SerDeError::MissingField("EdgeUpdate.forward_edge_name"))?
                .try_into()?,
            dst_uid: value
                .dst_uid
                .ok_or(SerDeError::MissingField("EdgeUpdate.reverse_edge_name"))?
                .try_into()?,
            forward_edge_name: value
                .forward_edge_name
                .ok_or(SerDeError::MissingField("EdgeUpdate.forward_edge_name"))?
                .try_into()?,
            reverse_edge_name: value
                .reverse_edge_name
                .ok_or(SerDeError::MissingField("EdgeUpdate.reverse_edge_name"))?
                .try_into()?,
        })
    }
}

impl From<EdgeUpdate> for EdgeUpdateProto {
    fn from(value: EdgeUpdate) -> Self {
        Self {
            src_uid: Some(value.src_uid.into()),
            dst_uid: Some(value.dst_uid.into()),
            forward_edge_name: Some(value.forward_edge_name.into()),
            reverse_edge_name: Some(value.reverse_edge_name.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Update {
    StringProperty(StringPropertyUpdate),
    Uint64Property(UInt64PropertyUpdate),
    Int64Property(Int64PropertyUpdate),
    Edge(EdgeUpdate),
}

impl TryFrom<UpdateProto> for Update {
    type Error = SerDeError;
    fn try_from(value: UpdateProto) -> Result<Self, Self::Error> {
        match value.inner {
            Some(UpdateInnerProto::StringProperty(update)) => {
                Ok(Update::StringProperty(update.try_into()?))
            }
            Some(UpdateInnerProto::Uint64Property(update)) => {
                Ok(Update::Uint64Property(update.try_into()?))
            }
            Some(UpdateInnerProto::Int64Property(update)) => {
                Ok(Update::Int64Property(update.try_into()?))
            }
            Some(UpdateInnerProto::Edge(update)) => Ok(Update::Edge(update.try_into()?)),
            None => Err(SerDeError::UnknownVariant("Update")),
        }
    }
}

impl From<Update> for UpdateProto {
    fn from(value: Update) -> Self {
        match value {
            Update::StringProperty(update) => UpdateProto {
                inner: Some(UpdateInnerProto::StringProperty(update.into())),
            },
            Update::Uint64Property(update) => UpdateProto {
                inner: Some(UpdateInnerProto::Uint64Property(update.into())),
            },
            Update::Int64Property(update) => UpdateProto {
                inner: Some(UpdateInnerProto::Int64Property(update.into())),
            },
            Update::Edge(update) => UpdateProto {
                inner: Some(UpdateInnerProto::Edge(update.into())),
            },
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Updates {
    pub updates: Vec<Update>,
}

impl Updates {
    pub fn new() -> Self {
        Self { updates: vec![] }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            updates: Vec::with_capacity(capacity),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Update> {
        self.updates.iter()
    }
}

impl IntoIterator for Updates {
    type Item = Update;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.updates.into_iter()
    }
}

impl TryFrom<UpdatesProto> for Updates {
    type Error = SerDeError;
    fn try_from(value: UpdatesProto) -> Result<Self, Self::Error> {
        Ok(Self {
            updates: value
                .updates
                .into_iter()
                .map(|update| update.try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<Updates> for UpdatesProto {
    fn from(value: Updates) -> Self {
        Self {
            updates: value
                .updates
                .into_iter()
                .map(|update| update.into())
                .collect(),
        }
    }
}

impl type_url::TypeUrl for Updates {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.Updates";
}

impl ProtobufSerializable for Updates {
    type ProtobufMessage = UpdatesProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LensRef {
    pub lens_namespace: String,
    pub lens_name: String,
}

impl TryFrom<LensRefProto> for LensRef {
    type Error = SerDeError;
    fn try_from(value: LensRefProto) -> Result<Self, Self::Error> {
        // todo: ensure lens_namespace and lens_name conform
        Ok(LensRef {
            lens_namespace: value.lens_namespace,
            lens_name: value.lens_name,
        })
    }
}

impl From<LensRef> for LensRefProto {
    fn from(value: LensRef) -> Self {
        LensRefProto {
            lens_namespace: value.lens_namespace,
            lens_name: value.lens_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnalyzerName {
    pub value: String,
}

impl TryFrom<AnalyzerNameProto> for AnalyzerName {
    type Error = SerDeError;
    fn try_from(value: AnalyzerNameProto) -> Result<Self, Self::Error> {
        // todo: Add check for length/ conformance defined in proto
        Ok(AnalyzerName { value: value.value })
    }
}

impl From<AnalyzerName> for AnalyzerNameProto {
    fn from(value: AnalyzerName) -> Self {
        AnalyzerNameProto { value: value.value }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionHit {
    pub graph_view: GraphView,
    pub lens_refs: Vec<LensRef>,
    pub analyzer_name: AnalyzerName,
    pub time_of_match: SystemTime,
    pub idempotency_key: u64,
    pub score: i32,
}

impl TryFrom<ExecutionHitProto> for ExecutionHit {
    type Error = SerDeError;
    fn try_from(value: ExecutionHitProto) -> Result<Self, Self::Error> {
        // todo: Add check for length/ conformance defined in proto
        Ok(Self {
            graph_view: value
                .graph_view
                .ok_or(SerDeError::MissingField("ExecutionHit.graph_view"))?
                .try_into()?,
            lens_refs: value
                .lens_refs
                .into_iter()
                .map(|lens_ref| lens_ref.try_into())
                .collect::<Result<Vec<_>, _>>()?,
            analyzer_name: value
                .analyzer_name
                .ok_or(SerDeError::MissingField("ExecutionHit.analyzer_name"))?
                .try_into()?,
            time_of_match: value
                .time_of_match
                .ok_or(SerDeError::MissingField("ExecutionHit.time_of_match"))?
                .try_into()?,
            idempotency_key: value.idempotency_key,
            score: value.score,
        })
    }
}

impl From<ExecutionHit> for ExecutionHitProto {
    fn from(value: ExecutionHit) -> Self {
        Self {
            graph_view: Some(value.graph_view.into()),
            lens_refs: value
                .lens_refs
                .into_iter()
                .map(|lens_ref| lens_ref.into())
                .collect(),
            analyzer_name: Some(value.analyzer_name.into()),
            time_of_match: Some(value.time_of_match.try_into().unwrap()), // this can never actually fail
            idempotency_key: value.idempotency_key.into(),
            score: value.score.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionMiss {}

impl TryFrom<ExecutionMissProto> for ExecutionMiss {
    type Error = SerDeError;
    fn try_from(value: ExecutionMissProto) -> Result<Self, Self::Error> {
        let ExecutionMissProto {} = value;
        Ok(Self {})
    }
}

impl From<ExecutionMiss> for ExecutionMissProto {
    fn from(value: ExecutionMiss) -> Self {
        let ExecutionMiss {} = value;
        Self {}
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    ExecutionHit(ExecutionHit),
    ExecutionMiss(ExecutionMiss),
}

impl TryFrom<ExecutionResultProto> for ExecutionResult {
    type Error = SerDeError;
    fn try_from(value: ExecutionResultProto) -> Result<Self, Self::Error> {
        match value.inner {
            Some(ExecutionResultInnerProto::Hit(inner)) => {
                Ok(Self::ExecutionHit(inner.try_into()?))
            }
            Some(ExecutionResultInnerProto::Miss(inner)) => {
                Ok(Self::ExecutionMiss(inner.try_into()?))
            }
            None => Err(SerDeError::UnknownVariant("ExecutionResult")),
        }
    }
}

impl From<ExecutionResult> for ExecutionResultProto {
    fn from(value: ExecutionResult) -> Self {
        match value {
            ExecutionResult::ExecutionHit(value) => Self {
                inner: Some(ExecutionResultInnerProto::Hit(value.into())),
            },
            ExecutionResult::ExecutionMiss(value) => Self {
                inner: Some(ExecutionResultInnerProto::Miss(value.into())),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunAnalyzerRequest {
    pub tenant_id: uuid::Uuid,
    pub update: Update,
}

impl TryFrom<RunAnalyzerRequestProto> for RunAnalyzerRequest {
    type Error = SerDeError;
    fn try_from(value: RunAnalyzerRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: value
                .tenant_id
                .ok_or(SerDeError::MissingField("RunAnalyzerRequest.tenant_id"))?
                .try_into()?,
            update: value
                .update
                .ok_or(SerDeError::MissingField("RunAnalyzerRequest.update"))?
                .try_into()?,
        })
    }
}

impl From<RunAnalyzerRequest> for RunAnalyzerRequestProto {
    fn from(value: RunAnalyzerRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            update: Some(value.update.into()),
        }
    }
}

impl type_url::TypeUrl for RunAnalyzerRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.RunAnalyzerRequest";
}

impl ProtobufSerializable for RunAnalyzerRequest {
    type ProtobufMessage = RunAnalyzerRequestProto;
}

#[derive(Debug, Clone)]
pub struct RunAnalyzerResponse {
    pub execution_result: ExecutionResult,
}

impl TryFrom<RunAnalyzerResponseProto> for RunAnalyzerResponse {
    type Error = SerDeError;
    fn try_from(value: RunAnalyzerResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            execution_result: value
                .execution_result
                .ok_or(SerDeError::MissingField(
                    "RunAnalyzerResponse.execution_result",
                ))?
                .try_into()?,
        })
    }
}

impl From<RunAnalyzerResponse> for RunAnalyzerResponseProto {
    fn from(value: RunAnalyzerResponse) -> Self {
        Self {
            execution_result: Some(value.execution_result.into()),
        }
    }
}

impl type_url::TypeUrl for RunAnalyzerResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.RunAnalyzerResponse";
}

impl ProtobufSerializable for RunAnalyzerResponse {
    type ProtobufMessage = RunAnalyzerResponseProto;
}
