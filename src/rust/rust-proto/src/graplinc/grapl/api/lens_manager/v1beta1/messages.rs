use crate::{
    graplinc::{
        common::v1beta1::Uuid,
        grapl::common::v1beta1::types::{
            NodeType,
            Uid,
        },
    },
    protobufs::graplinc::grapl::api::lens_manager::v1beta1::{
        merge_lens_request::MergeBehavior as MergeBehaviorProto,
        AddNodeToScopeRequest as AddNodeToScopeRequestProto,
        AddNodeToScopeResponse as AddNodeToScopeResponseProto,
        CloseLensRequest as CloseLensRequestProto,
        CloseLensResponse as CloseLensResponseProto,
        CreateLensRequest as CreateLensRequestProto,
        CreateLensResponse as CreateLensResponseProto,
        ListLensesEntry as ListLensesEntryProto,
        ListLensesRequest as ListLensesRequestProto,
        ListLensesResponse as ListLensesResponseProto,
        MergeLensRequest as MergeLensRequestProto,
        MergeLensResponse as MergeLensResponseProto,
        RemoveNodeFromAllScopesRequest as RemoveNodeFromAllScopesRequestProto,
        RemoveNodeFromAllScopesResponse as RemoveNodeFromAllScopesResponseProto,
        RemoveNodeFromScopeRequest as RemoveNodeFromScopeRequestProto,
        RemoveNodeFromScopeResponse as RemoveNodeFromScopeResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// CreateLensRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct CreateLensRequest {
    pub tenant_id: Uuid,
    pub lens_type: String,
    pub lens_name: String,
    pub is_engagement: bool,
}

impl TryFrom<CreateLensRequestProto> for CreateLensRequest {
    type Error = SerDeError;

    fn try_from(request_proto: CreateLensRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        let lens_type = request_proto.lens_type;
        let lens_name = request_proto.lens_name;
        let is_engagement = request_proto.is_engagement;

        Ok(CreateLensRequest {
            tenant_id: tenant_id.into(),
            lens_type,
            lens_name,
            is_engagement,
        })
    }
}

impl From<CreateLensRequest> for CreateLensRequestProto {
    fn from(request: CreateLensRequest) -> Self {
        CreateLensRequestProto {
            tenant_id: Some(request.tenant_id.into()),
            lens_type: request.lens_type,
            lens_name: request.lens_name,
            is_engagement: request.is_engagement,
        }
    }
}

impl type_url::TypeUrl for CreateLensRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CreateLensRequest";
}

impl serde_impl::ProtobufSerializable for CreateLensRequest {
    type ProtobufMessage = CreateLensRequestProto;
}

// //
// // CreateLensResponse
// //

#[derive(Debug, Clone, PartialEq)]
pub struct CreateLensResponse {
    pub lens_uid: u64,
}

impl TryFrom<CreateLensResponseProto> for CreateLensResponse {
    type Error = SerDeError;

    fn try_from(response_proto: CreateLensResponseProto) -> Result<Self, Self::Error> {
        Ok(CreateLensResponse {
            lens_uid: response_proto.lens_uid,
        })
    }
}

impl From<CreateLensResponse> for CreateLensResponseProto {
    fn from(response: CreateLensResponse) -> Self {
        CreateLensResponseProto {
            lens_uid: response.lens_uid,
        }
    }
}

impl type_url::TypeUrl for CreateLensResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CreateLensResponse";
}

impl serde_impl::ProtobufSerializable for CreateLensResponse {
    type ProtobufMessage = CreateLensResponseProto;
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergeBehavior {
    Preserve,
    Close,
}

impl TryFrom<MergeBehaviorProto> for MergeBehavior {
    type Error = SerDeError;

    fn try_from(merge_behavior: MergeBehaviorProto) -> Result<Self, Self::Error> {
        match merge_behavior {
            MergeBehaviorProto::Unspecified => Err(SerDeError::UnknownVariant("Unspecified")),
            MergeBehaviorProto::Preserve => Ok(MergeBehavior::Preserve),
            MergeBehaviorProto::Close => Ok(MergeBehavior::Close),
        }
    }
}

impl From<MergeBehavior> for MergeBehaviorProto {
    fn from(merge_behavior: MergeBehavior) -> Self {
        match merge_behavior {
            MergeBehavior::Preserve => MergeBehaviorProto::Preserve,
            MergeBehavior::Close => MergeBehaviorProto::Close,
        }
    }
}

impl type_url::TypeUrl for MergeBehavior {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.MergeBehavior";
}

//
// MergeLensRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct MergeLensRequest {
    pub tenant_id: Uuid,
    pub source_lens_uid: u64,
    pub target_lens_uid: u64,
    pub merge_behavior: MergeBehavior,
}

impl TryFrom<MergeLensRequestProto> for MergeLensRequest {
    type Error = SerDeError;

    fn try_from(request_proto: MergeLensRequestProto) -> Result<Self, Self::Error> {
        let merge_behavior = request_proto.merge_behavior().try_into()?;

        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();

        let source_lens_uid = request_proto.source_lens_uid;

        let target_lens_uid = request_proto.target_lens_uid;

        Ok(MergeLensRequest {
            tenant_id,
            source_lens_uid,
            target_lens_uid,
            merge_behavior,
        })
    }
}

impl From<MergeLensRequest> for MergeLensRequestProto {
    fn from(request: MergeLensRequest) -> Self {
        MergeLensRequestProto {
            tenant_id: Some(request.tenant_id.into()),
            source_lens_uid: request.source_lens_uid.into(),
            target_lens_uid: request.target_lens_uid.into(),
            merge_behavior: request.merge_behavior as i32,
        }
    }
}

impl type_url::TypeUrl for MergeLensRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.MergeLensRequest";
}

impl serde_impl::ProtobufSerializable for MergeLensRequest {
    type ProtobufMessage = MergeLensRequestProto;
}

//
// MergeLensResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct MergeLensResponse {}

impl TryFrom<MergeLensResponseProto> for MergeLensResponse {
    type Error = SerDeError;
    fn try_from(_response_proto: MergeLensResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<MergeLensResponse> for MergeLensResponseProto {
    fn from(_request: MergeLensResponse) -> Self {
        MergeLensResponseProto {}
    }
}

impl type_url::TypeUrl for MergeLensResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.MergeLensResponse";
}

impl serde_impl::ProtobufSerializable for MergeLensResponse {
    type ProtobufMessage = MergeLensResponseProto;
}

//
//  CloseLensRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct CloseLensRequest {
    pub tenant_id: Uuid,
    pub lens_uid: u64,
}

impl TryFrom<CloseLensRequestProto> for CloseLensRequest {
    type Error = SerDeError;

    fn try_from(request_proto: CloseLensRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?
            .into();

        let lens_uid = request_proto.lens_uid;

        Ok(CloseLensRequest {
            tenant_id,
            lens_uid,
        })
    }
}

impl From<CloseLensRequest> for CloseLensRequestProto {
    fn from(request: CloseLensRequest) -> Self {
        CloseLensRequestProto {
            tenant_id: Some(request.tenant_id.into()),
            lens_uid: request.lens_uid,
        }
    }
}

impl type_url::TypeUrl for CloseLensRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensRequest";
}

impl serde_impl::ProtobufSerializable for CloseLensRequest {
    type ProtobufMessage = CloseLensRequestProto;
}

//
// CloseLensResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct CloseLensResponse {}

impl TryFrom<CloseLensResponseProto> for CloseLensResponse {
    type Error = SerDeError;

    fn try_from(_response_proto: CloseLensResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<CloseLensResponse> for CloseLensResponseProto {
    fn from(_request: CloseLensResponse) -> Self {
        CloseLensResponseProto {}
    }
}

impl type_url::TypeUrl for CloseLensResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensResponse";
}

impl serde_impl::ProtobufSerializable for CloseLensResponse {
    type ProtobufMessage = CloseLensResponseProto;
}

//
// AddNodeToScopeRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct AddNodeToScopeRequest {
    pub tenant_id: Uuid,
    pub lens_uid: u64,
    // replace
    pub uid: u64,
    //replace u64 with uid
    pub node_type: NodeType,
}

impl TryFrom<AddNodeToScopeRequestProto> for AddNodeToScopeRequest {
    type Error = SerDeError;

    fn try_from(request_proto: AddNodeToScopeRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        let lens_uid = request_proto.lens_uid;

        let uid = request_proto.uid;

        let node_type = request_proto
            .node_type
            .ok_or(SerDeError::MissingField("GetEdgeSchemaRequest.node_type"))?
            .try_into()?;

        Ok(AddNodeToScopeRequest {
            tenant_id: tenant_id.into(),
            lens_uid,
            uid,
            node_type,
        })
    }
}

impl From<AddNodeToScopeRequest> for AddNodeToScopeRequestProto {
    fn from(request: AddNodeToScopeRequest) -> Self {
        AddNodeToScopeRequestProto {
            tenant_id: Some(request.tenant_id.into()),
            lens_uid: request.lens_uid,
            uid: request.uid.into(),
            node_type: Some(request.node_type.into()),
        }
    }
}

impl type_url::TypeUrl for AddNodeToScopeRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.AddNodeToScopeRequest";
}

impl serde_impl::ProtobufSerializable for AddNodeToScopeRequest {
    type ProtobufMessage = AddNodeToScopeRequestProto;
}

// AddNodeToScopeResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct AddNodeToScopeResponse {}

impl TryFrom<AddNodeToScopeResponseProto> for AddNodeToScopeResponse {
    type Error = SerDeError;
    fn try_from(_response_proto: AddNodeToScopeResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<AddNodeToScopeResponse> for AddNodeToScopeResponseProto {
    fn from(_response: AddNodeToScopeResponse) -> Self {
        AddNodeToScopeResponseProto {}
    }
}

impl type_url::TypeUrl for AddNodeToScopeResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.AddNodeToScopeResponse";
}

impl serde_impl::ProtobufSerializable for AddNodeToScopeResponse {
    type ProtobufMessage = AddNodeToScopeResponseProto;
}

//
// RemoveNodeFromScopeRequest
//
#[derive(Debug, Clone, PartialEq)]
pub struct RemoveNodeFromScopeRequest {
    pub tenant_id: Uuid,
    pub lens_uid: u64,
    pub uid: u64,
}

impl TryFrom<RemoveNodeFromScopeRequestProto> for RemoveNodeFromScopeRequest {
    type Error = SerDeError;

    fn try_from(request_proto: RemoveNodeFromScopeRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        let lens_uid = request_proto.lens_uid;

        Ok(RemoveNodeFromScopeRequest {
            tenant_id: tenant_id.into(),
            lens_uid,
            uid: request_proto.uid,
        })
    }
}

impl From<RemoveNodeFromScopeRequest> for RemoveNodeFromScopeRequestProto {
    fn from(request: RemoveNodeFromScopeRequest) -> Self {
        RemoveNodeFromScopeRequestProto {
            tenant_id: Some(request.tenant_id.into()),
            lens_uid: request.lens_uid,
            uid: request.uid,
        }
    }
}

impl type_url::TypeUrl for RemoveNodeFromScopeRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromScopeRequest";
}

impl serde_impl::ProtobufSerializable for RemoveNodeFromScopeRequest {
    type ProtobufMessage = RemoveNodeFromScopeRequestProto;
}

//
// RemoveNodeFromScopeResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveNodeFromScopeResponse {}

impl TryFrom<RemoveNodeFromScopeResponseProto> for RemoveNodeFromScopeResponse {
    type Error = SerDeError;
    fn try_from(_response_proto: RemoveNodeFromScopeResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<RemoveNodeFromScopeResponse> for RemoveNodeFromScopeResponseProto {
    fn from(_request: RemoveNodeFromScopeResponse) -> Self {
        RemoveNodeFromScopeResponseProto {}
    }
}

impl type_url::TypeUrl for RemoveNodeFromScopeResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromScopeResponse";
}

impl serde_impl::ProtobufSerializable for RemoveNodeFromScopeResponse {
    type ProtobufMessage = RemoveNodeFromScopeResponseProto;
}

//
// RemoveNodeFromAllScopesRequest
//

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveNodeFromAllScopesRequest {
    pub tenant_id: Uuid,
    pub uid: u64,
}

impl TryFrom<RemoveNodeFromAllScopesRequestProto> for RemoveNodeFromAllScopesRequest {
    type Error = SerDeError;

    fn try_from(request_proto: RemoveNodeFromAllScopesRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        let uid = request_proto.uid;

        // check for invalid values for u64 and send back error otherwise deserailize
        Ok(RemoveNodeFromAllScopesRequest {
            tenant_id: tenant_id.into(),
            uid,
        })
    }
}

impl From<RemoveNodeFromAllScopesRequest> for RemoveNodeFromAllScopesRequestProto {
    fn from(request: RemoveNodeFromAllScopesRequest) -> Self {
        RemoveNodeFromAllScopesRequestProto {
            tenant_id: Some(request.tenant_id.into()),
            uid: request.uid.into(),
        }
    }
}

impl type_url::TypeUrl for RemoveNodeFromAllScopesRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromAllScopesRequest";
}

impl serde_impl::ProtobufSerializable for RemoveNodeFromAllScopesRequest {
    type ProtobufMessage = RemoveNodeFromAllScopesRequestProto;
}

//
// RemoveNodeFromAllScopesResponse
//

#[derive(Debug, Clone, PartialEq)]
pub struct RemoveNodeFromAllScopesResponse {}

impl TryFrom<RemoveNodeFromAllScopesResponseProto> for RemoveNodeFromAllScopesResponse {
    type Error = SerDeError;

    fn try_from(_response: RemoveNodeFromAllScopesResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

impl From<RemoveNodeFromAllScopesResponse> for RemoveNodeFromAllScopesResponseProto {
    fn from(_response: RemoveNodeFromAllScopesResponse) -> Self {
        RemoveNodeFromAllScopesResponseProto {}
    }
}

impl type_url::TypeUrl for RemoveNodeFromAllScopesResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromAllScopesResponse";
}

impl serde_impl::ProtobufSerializable for RemoveNodeFromAllScopesResponse {
    type ProtobufMessage = RemoveNodeFromAllScopesResponseProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListLensesRequest {
    pub tenant_id: Uuid,
    pub offset: u64,
    pub limit: u32,
}

impl TryFrom<ListLensesRequestProto> for ListLensesRequest {
    type Error = SerDeError;

    fn try_from(value: ListLensesRequestProto) -> Result<Self, Self::Error> {
        Ok(Self {
            tenant_id: value
                .tenant_id
                .ok_or(SerDeError::MissingField("tenant_id"))?
                .into(),
            offset: value.offset,
            limit: value.limit,
        })
    }
}

impl From<ListLensesRequest> for ListLensesRequestProto {
    fn from(value: ListLensesRequest) -> Self {
        Self {
            tenant_id: Some(value.tenant_id.into()),
            offset: value.offset,
            limit: value.limit,
        }
    }
}

impl type_url::TypeUrl for ListLensesRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.ListLensesRequest";
}

impl serde_impl::ProtobufSerializable for ListLensesRequest {
    type ProtobufMessage = ListLensesRequestProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListLensesEntry {
    pub lens_uid: Uid,
    // pub score: i64,
}

impl TryFrom<ListLensesEntryProto> for ListLensesEntry {
    type Error = SerDeError;

    fn try_from(value: ListLensesEntryProto) -> Result<Self, Self::Error> {
        Ok(Self {
            lens_uid: value
                .lens_uid
                .ok_or(SerDeError::MissingField("lens_uid"))?
                .try_into()?,
            // score: value.score,
        })
    }
}

impl From<ListLensesEntry> for ListLensesEntryProto {
    fn from(value: ListLensesEntry) -> Self {
        Self {
            lens_uid: Some(value.lens_uid.into()),
            // score: value.score,
        }
    }
}

impl type_url::TypeUrl for ListLensesEntry {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.ListLensesEntryProto";
}

impl serde_impl::ProtobufSerializable for ListLensesEntry {
    type ProtobufMessage = ListLensesEntryProto;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListLensesResponse {
    pub lenses: Vec<ListLensesEntry>,
    pub offset: u64,
}

impl TryFrom<ListLensesResponseProto> for ListLensesResponse {
    type Error = SerDeError;

    fn try_from(response_proto: ListLensesResponseProto) -> Result<Self, Self::Error> {
        Ok(Self {
            lenses: response_proto
                .lenses
                .into_iter()
                .map(ListLensesEntry::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            offset: response_proto.offset,
        })
    }
}

impl From<ListLensesResponse> for ListLensesResponseProto {
    fn from(response: ListLensesResponse) -> Self {
        Self {
            lenses: response
                .lenses
                .into_iter()
                .map(ListLensesEntry::into)
                .collect(),
            offset: response.offset,
        }
    }
}

impl type_url::TypeUrl for ListLensesResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.ListLensesResponse";
}

impl serde_impl::ProtobufSerializable for ListLensesResponse {
    type ProtobufMessage = ListLensesResponseProto;
}
