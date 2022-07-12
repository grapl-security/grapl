use std::time::SystemTime;
use prost_types::Timestamp;
use crate::graplinc::grapl::api::graph_query_service::v1beta1::messages::GraphView;
use crate::graplinc::grapl::common::v1beta1::types::{EdgeName, NodeType, PropertyName, Uid};
use crate::protobufs::graplinc::grapl::api::plugin_sdk::analyzers::v1beta1::{
    AnalyzerName as AnalyzerNameProto,
    EdgeUpdate as EdgeUpdateProto,
    StringPropertyUpdate as StringPropertyUpdateProto,
    UInt64PropertyUpdate as UInt64PropertyUpdateProto,
    Int64PropertyUpdate as Int64PropertyUpdateProto,
    Update as UpdateProto,
    Updates as UpdatesProto,
    LensRef as LensRefProto,
    ExecutionHit as ExecutionHitProto,
    ExecutionMiss as ExecutionMissProto,
    ExecutionResult as ExecutionResultProto,
    RunAnalyzerRequest as RunAnalyzerRequestProto,
    RunAnalyzerResponse as RunAnalyzerResponseProto,
    update::Inner as UpdateInnerProto,
    execution_result::Inner as ExecutionResultInnerProto,
};
use crate::{serde_impl, SerDeError, type_url};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringPropertyUpdate {
    pub uid: Uid,
    pub property_name: PropertyName,
}

impl TryFrom<StringPropertyUpdateProto> for StringPropertyUpdate {
    type Error = SerDeError;
    fn try_from(value: StringPropertyUpdateProto) -> Result<Self, Self::Error> {
        Ok(Self {
            uid: value.uid.ok_or(SerDeError::MissingField("uid"))?.try_into()?,
            property_name: value.property_name.ok_or(SerDeError::MissingField("property_name"))?.try_into()?,
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
            uid: value.uid.ok_or(SerDeError::MissingField("uid"))?.try_into()?,
            property_name: value.property_name.ok_or(SerDeError::MissingField("property_name"))?.try_into()?,
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
            uid: value.uid.ok_or(SerDeError::MissingField("uid"))?.try_into()?,
            property_name: value.property_name.ok_or(SerDeError::MissingField("property_name"))?.try_into()?,
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
            Some(UpdateInnerProto::StringPropertyUpdate(update)) => Ok(Update::StringPropertyUpdate(update.try_into()?)),
            Some(UpdateInnerProto::Uint64PropertyUpdate(update)) => Ok(Update::Uint64PropertyUpdate(update.try_into()?)),
            Some(UpdateInnerProto::Int64PropertyUpdate(update)) => Ok(Update::Int64PropertyUpdate(update.try_into()?)),
            Some(UpdateInnerProto::EdgeUpdate(update)) => Ok(Update::EdgeUpdate(update.try_into()?)),
            None => Err(SerDeError::UnknownVariant("Update")),
        }
    }
}


impl From<Update> for UpdateProto {
    fn from(value: Update) -> Self {
        match value {
            Update::StringPropertyUpdate(update) => UpdateProto{inner: Some(UpdateInnerProto::StringPropertyUpdate(update.into()))},
            Update::Uint64PropertyUpdate(update) => UpdateProto{inner: Some(UpdateInnerProto::Uint64PropertyUpdate(update.into()))},
            Update::Int64PropertyUpdate(update) => UpdateProto{inner: Some(UpdateInnerProto::Int64PropertyUpdate(update.into()))},
            Update::EdgeUpdate(update) => UpdateProto{inner: Some(UpdateInnerProto::EdgeUpdate(update.into()))},
        }
    }
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
        Ok(Self {
            updates,
        })
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
    const TYPE_URL: &'static str = "graplsecurity.com/graplinc.grapl.api.plugin_sdk.analyzers.v1beta1.Updates";
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnalyzerName {
    pub value: String,
}


impl TryFrom<AnalyzerNameProto> for AnalyzerName {
    type Error = SerDeError;
    fn try_from(value: AnalyzerNameProto) -> Result<Self, Self::Error> {
        Ok(AnalyzerName {
            value: value.value,
        })
    }
}

impl From<AnalyzerName> for AnalyzerNameProto {
    fn from(value: AnalyzerName) -> Self {
        AnalyzerNameProto {
            value: value.value,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionMiss {}

#[derive(Debug, Clone)]
pub enum ExecutionResult {
    ExecutionHit(ExecutionHit),
    ExecutionMiss(ExecutionMiss),
}

#[derive(Debug, Clone)]
pub struct RunAnalyzerRequest {
    pub tenant_id: uuid::Uuid,
    pub update: Update,
}

impl TryFrom<RunAnalyzerRequestProto> for RunAnalyzerRequest {
    type Error = SerDeError;
    fn try_from(value: RunAnalyzerRequestProto) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<RunAnalyzerRequest> for RunAnalyzerRequestProto {
    fn from(value: RunAnalyzerRequest) -> Self {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct RunAnalyzerResponse {
    pub execution_result: ExecutionResult,
}


impl TryFrom<RunAnalyzerResponseProto> for RunAnalyzerResponse {
    type Error = SerDeError;
    fn try_from(value: RunAnalyzerResponseProto) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<RunAnalyzerResponse> for RunAnalyzerResponseProto {
    fn from(value: RunAnalyzerResponse) -> Self {
        todo!()
    }
}










