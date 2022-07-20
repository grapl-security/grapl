use std::time::SystemTime;

use crate::{
    graplinc::grapl::{
        api::graph_query_service::v1beta1::messages::GraphView,
        common::v1beta1::types::{
            EdgeName,
            NodeType,
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
    serde_impl,
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

impl type_url::TypeUrl for StringPropertyUpdate {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.StringPropertyUpdate";
}

impl serde_impl::ProtobufSerializable for StringPropertyUpdate {
    type ProtobufMessage = StringPropertyUpdateProto;
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

impl type_url::TypeUrl for UInt64PropertyUpdate {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.UInt64PropertyUpdate";
}

impl serde_impl::ProtobufSerializable for UInt64PropertyUpdate {
    type ProtobufMessage = UInt64PropertyUpdateProto;
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

impl type_url::TypeUrl for Int64PropertyUpdate {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.Int64PropertyUpdate";
}

impl serde_impl::ProtobufSerializable for Int64PropertyUpdate {
    type ProtobufMessage = Int64PropertyUpdateProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeUpdate {
    pub src_edge_name: EdgeName,
    pub src_node_type: NodeType,
    pub src_uid: Uid,
    pub dst_uid: Uid,
}

impl TryFrom<EdgeUpdateProto> for EdgeUpdate {
    type Error = SerDeError;

    fn try_from(value: EdgeUpdateProto) -> Result<Self, Self::Error> {
        Ok(Self {
            src_edge_name: value
                .src_edge_name
                .ok_or_else(|| SerDeError::MissingField("src_edge_name"))?
                .try_into()?,
            src_node_type: value
                .src_node_type
                .ok_or_else(|| SerDeError::MissingField("src_node_type"))?
                .try_into()?,
            src_uid: value
                .src_uid
                .ok_or_else(|| SerDeError::MissingField("src_uid"))?
                .try_into()?,
            dst_uid: value
                .dst_uid
                .ok_or_else(|| SerDeError::MissingField("dst_uid"))?
                .try_into()?,
        })
    }
}

impl From<EdgeUpdate> for EdgeUpdateProto {
    fn from(value: EdgeUpdate) -> Self {
        Self {
            src_edge_name: Some(value.src_edge_name.into()),
            src_node_type: Some(value.src_node_type.into()),
            src_uid: Some(value.src_uid.into()),
            dst_uid: Some(value.dst_uid.into()),
        }
    }
}

impl type_url::TypeUrl for EdgeUpdate {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.EdgeUpdate";
}

impl serde_impl::ProtobufSerializable for EdgeUpdate {
    type ProtobufMessage = EdgeUpdateProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Update {
    StringPropertyUpdate(StringPropertyUpdate),
    Uint64PropertyUpdate(UInt64PropertyUpdate),
    Int64PropertyUpdate(Int64PropertyUpdate),
    EdgeUpdate(EdgeUpdate),
}

impl TryFrom<UpdateProto> for Update {
    type Error = SerDeError;
    fn try_from(value: UpdateProto) -> Result<Self, Self::Error> {
        match value.inner {
            Some(UpdateInnerProto::StringPropertyUpdate(update)) => {
                Ok(Update::StringPropertyUpdate(update.try_into()?))
            }
            Some(UpdateInnerProto::Uint64PropertyUpdate(update)) => {
                Ok(Update::Uint64PropertyUpdate(update.try_into()?))
            }
            Some(UpdateInnerProto::Int64PropertyUpdate(update)) => {
                Ok(Update::Int64PropertyUpdate(update.try_into()?))
            }
            Some(UpdateInnerProto::EdgeUpdate(update)) => {
                Ok(Update::EdgeUpdate(update.try_into()?))
            }
            None => Err(SerDeError::UnknownVariant("Update")),
        }
    }
}

impl From<Update> for UpdateProto {
    fn from(value: Update) -> Self {
        match value {
            Update::StringPropertyUpdate(update) => UpdateProto {
                inner: Some(UpdateInnerProto::StringPropertyUpdate(update.into())),
            },
            Update::Uint64PropertyUpdate(update) => UpdateProto {
                inner: Some(UpdateInnerProto::Uint64PropertyUpdate(update.into())),
            },
            Update::Int64PropertyUpdate(update) => UpdateProto {
                inner: Some(UpdateInnerProto::Int64PropertyUpdate(update.into())),
            },
            Update::EdgeUpdate(update) => UpdateProto {
                inner: Some(UpdateInnerProto::EdgeUpdate(update.into())),
            },
        }
    }
}

impl type_url::TypeUrl for Update {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.Update";
}

impl serde_impl::ProtobufSerializable for Update {
    type ProtobufMessage = UpdateProto;
}

#[derive(Debug, Clone)]
pub struct Updates {
    pub updates: Vec<Update>,
}

impl TryFrom<UpdatesProto> for Updates {
    type Error = SerDeError;
    fn try_from(value: UpdatesProto) -> Result<Self, Self::Error> {
        let updates = value
            .updates
            .into_iter()
            .map(|update| update.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { updates })
    }
}

impl From<Updates> for UpdatesProto {
    fn from(value: Updates) -> Self {
        Self {
            updates: value.updates.into_iter().map(UpdateProto::from).collect(),
        }
    }
}

impl type_url::TypeUrl for Updates {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.Updates";
}

impl serde_impl::ProtobufSerializable for Updates {
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

impl type_url::TypeUrl for LensRef {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.LensRef";
}

impl serde_impl::ProtobufSerializable for LensRef {
    type ProtobufMessage = LensRefProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnalyzerName {
    pub value: String,
}

impl TryFrom<AnalyzerNameProto> for AnalyzerName {
    type Error = SerDeError;
    fn try_from(value: AnalyzerNameProto) -> Result<Self, Self::Error> {
        Ok(AnalyzerName { value: value.value })
    }
}

impl From<AnalyzerName> for AnalyzerNameProto {
    fn from(value: AnalyzerName) -> Self {
        AnalyzerNameProto { value: value.value }
    }
}

impl type_url::TypeUrl for AnalyzerName {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.AnalyzerName";
}

impl serde_impl::ProtobufSerializable for AnalyzerName {
    type ProtobufMessage = AnalyzerNameProto;
}

#[derive(Debug, Clone)]
pub struct ExecutionHit {
    pub graph_view: GraphView,
    pub root_uid: Uid,
    pub lens_refs: Vec<LensRef>,
    pub analyzer_name: AnalyzerName,
    pub time_of_match: SystemTime,
    pub idempotency_key: u64,
    pub score: i32,
}

impl TryFrom<ExecutionHitProto> for ExecutionHit {
    type Error = SerDeError;
    fn try_from(value: ExecutionHitProto) -> Result<Self, Self::Error> {
        Ok(Self {
            graph_view: value
                .graph_view
                .ok_or(SerDeError::MissingField("graph_view"))?
                .try_into()?,
            root_uid: value
                .root_uid
                .ok_or(SerDeError::MissingField("root_uid"))?
                .try_into()?,
            lens_refs: value
                .lens_refs
                .into_iter()
                .map(LensRef::try_from)
                .collect::<Result<_, _>>()?,
            analyzer_name: value
                .analyzer_name
                .ok_or(SerDeError::MissingField("analyzer_name"))?
                .try_into()?,
            time_of_match: value
                .time_of_match
                .ok_or(SerDeError::MissingField("time_of_match"))?
                .try_into()?,
            idempotency_key: value.idempotency_key,
            score: value.score,
        })
    }
}

impl From<ExecutionHit> for ExecutionHitProto {
    fn from(value: ExecutionHit) -> Self {
        ExecutionHitProto {
            graph_view: Some(value.graph_view.into()),
            root_uid: Some(value.root_uid.into()),
            lens_refs: value.lens_refs.into_iter().map(LensRef::into).collect(),
            analyzer_name: Some(value.analyzer_name.into()),
            time_of_match: Some(value.time_of_match.try_into().unwrap()), // This can't actually fail
            idempotency_key: value.idempotency_key,
            score: value.score,
        }
    }
}

impl type_url::TypeUrl for ExecutionHit {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.ExecutionHit";
}

impl serde_impl::ProtobufSerializable for ExecutionHit {
    type ProtobufMessage = ExecutionHitProto;
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
        ExecutionMissProto {}
    }
}

impl type_url::TypeUrl for ExecutionMiss {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.ExecutionMiss";
}

impl serde_impl::ProtobufSerializable for ExecutionMiss {
    type ProtobufMessage = ExecutionMissProto;
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
            Some(ExecutionResultInnerProto::Hit(hit)) => {
                Ok(ExecutionResult::ExecutionHit(hit.try_into()?))
            }
            Some(ExecutionResultInnerProto::Miss(_miss)) => {
                Ok(ExecutionResult::ExecutionMiss(ExecutionMiss {}))
            }
            None => Err(SerDeError::UnknownVariant("ExecutionResult")),
        }
    }
}

impl From<ExecutionResult> for ExecutionResultProto {
    fn from(value: ExecutionResult) -> ExecutionResultProto {
        match value {
            ExecutionResult::ExecutionHit(hit) => ExecutionResultProto {
                inner: Some(ExecutionResultInnerProto::Hit(hit.into())),
            },
            ExecutionResult::ExecutionMiss(miss) => ExecutionResultProto {
                inner: Some(ExecutionResultInnerProto::Miss(miss.into())),
            },
        }
    }
}

impl type_url::TypeUrl for ExecutionResult {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.ExecutionResult";
}

impl serde_impl::ProtobufSerializable for ExecutionResult {
    type ProtobufMessage = ExecutionResultProto;
}

#[derive(Debug, Clone)]
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
                .ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            update: value
                .update
                .ok_or(SerDeError::MissingField("update"))?
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

impl serde_impl::ProtobufSerializable for RunAnalyzerRequest {
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
                .ok_or(SerDeError::MissingField("execution_result"))?
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

impl serde_impl::ProtobufSerializable for RunAnalyzerResponse {
    type ProtobufMessage = RunAnalyzerResponseProto;
}
