use crate::{
    graplinc::common::v1beta1::{
        Uuid
    },
    protobufs::graplinc::grapl::api::lens_manager::v1beta1::{
        CreateLensRequest as CreateLensRequestProto,
        CreateLensResponse as CreateLensResponseProto,
        // MergeLensRequest as MergeLensRequestProto,
        // MergeLensResponse as MergeLensResponseProto,
        // CloseLensRequest as CloseLensRequestProto,
        // CloseLensResponse as CloseLensResponseProto,
        // AddNodeToScopeRequest as AddNodeToScopeRequestProto,
        // AddNodeToScopeResponse as AddNodeToScopeResponseProto,
        // RemoveNodeFromScopeRequest as RemoveNodeFromScopeRequestProto,
        // RemoveNodeFromScopeResponse as RemoveNodeFromScopeResponseProto,
        // RemoveNodeFromAllScopesRequest as RemoveNodeFromAllScopesRequestProto,
        // RemoveNodeFromAllScopesResponse as RemoveNodeFromAllScopesResponseProto,
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
    pub key: String,
    pub value: String,
    pub is_valid: bool,
}

impl TryFrom<CreateLensRequestProto> for CreateLensRequest {
    type Error = SerDeError;

    fn try_from(request_proto: CreateLensRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"))?;

        let key = request_proto.key;
        let value = request_proto.value;
        let is_valid = request_proto.is_valid;

        Ok(CreateLensRequest {
            tenant_id: tenant_id.into(),
            key,
            value,
            is_valid,
        })
    }
}

impl From<CreateLensRequest> for CreateLensRequestProto {
    fn from(request: CreateLensRequest) -> Self {
        CreateLensRequestProto {
            tenant_id: Some(request.tenant_id.into()),
            key: request.key,
            value: request.value,
            is_valid: request.is_valid,
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

impl From<CreateLensResponse> for CreateLensResponseProto{
    fn from(response: CreateLensResponse) -> Self {
        CreateLensResponseProto{
            lens_uid: response.lens_uid,
        }
    }
}

impl type_url::TypeUrl for CreateLensResponse{
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CreateLensResponse";
}

impl serde_impl::ProtobufSerializable for CreateLensResponse {
    type ProtobufMessage = CreateLensResponseProto;
}

//
// //
// // MergeLensRequest
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct MergeLensRequest {
//     pub tenant_id: Uuid,
//     pub source_lens_uid: u64,
//     pub target_lens_uid: u64,
//     pub close_source: bool,
//
// }
//
// impl TryFrom<MergeLensRequestProto> for MergeLensRequest{
//     type Error = SerDeError;
//
//     fn try_from(request_proto: MergeLensRequestProto) -> Result<Self, Self::Error> {
//         let tenant_id = request_proto
//             .tenant_id
//             .ok_or(SerDeError::MissingField("tenant_id"))?;
//
//         let source_lens_uid = request_proto
//             .source_lens_uid
//             .ok_or(SerDeError::MissingField("source_lens_uid"))?;
//
//         let target_lens_uid = request_proto
//             .target_lens_uid
//             .ok_or(SerDeError::MissingField("target_lens_uid"))?;
//
//         let close_source = request_proto
//             .close_source
//             .ok_or(SerDeError::MissingField("close_source"))?;
//
//         Ok(MergeLensRequest {
//             tenant_id: tenant_id.into(),
//             source_lens_uid: source_lens_uid.into(),
//             target_lens_uid: target_lens_uid.into(),
//             close_source: close_source.into(),
//         })
//     }
// }
//
// impl From<MergeLensRequest> for MergeLensRequestProto {
//     fn from(request: MergeLensRequest) -> Self {
//         MergeLensRequestProto{
//             tenant_id: Some(request.tenant_id.into()),
//             source_lens_uid: Some(request.source_lens_uid.into()),
//             target_lens_uid: Some(request.target_lens_uid.into()),
//             close_source: Some(request.close_source.into()),
//         }
//     }
// }
//
// impl type_url::TypeUrl for MergeLensRequest{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.MergeLensRequest";
// }
//
// impl serde_impl::ProtobufSerializable for MergeLensRequest {
//     type ProtobufMessage = MergeLensRequestProto;
// }
//
// //
// // MergeLensResponse
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct MergeLensResponse {
//
// }
//
// impl TryFrom<MergeLensResponseProto> for MergeLensResponse {
//     type Error = SerDeError;
//     fn try_from(response_proto: MergeLensResponseProto) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl TryFrom<MergeLensResponse> for MergeLensResponseProto{
//     type Error = SerDeError;
//
//     fn try_from(response: MergeLensResponse) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl type_url::TypeUrl for MergeLensResponse{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.MergeLensResponse";
// }
//
// impl serde_impl::ProtobufSerializable for MergeLensResponse {
//     type ProtobufMessage = MergeLensResponseProto;
// }
//
// //
// //  CloseLensRequest
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct CloseLensRequest {
//     pub tenant_id: Uuid,
//     pub lens_uid: u64,
// }
//
// impl TryFrom<CloseLensRequestProto> for CloseLensRequest {
//     type Error = SerDeError;
//
//     fn try_from(request_proto: CloseLensRequestProto) -> Result<Self, Self::Error>{
//         let tenant_id = request_proto
//             .tenant_id
//             .ok_or(SerDeError::MissingField("tenant_id"))?;
//
//         let lens_uid = request_proto
//             .lens_uid
//             .ok_or(SerDeError::MissingField("lens_uid"))?;
//
//         Ok(CloseLensRequest{
//             tenant_id: tenant_id.into(),
//             lens_uid: lens_uid.into(),
//         })
//     }
// }
//
// impl From<CloseLensRequest> for CloseLensRequestProto {
//     fn from(request: CloseLensRequest) -> Self {
//         CloseLensRequestProto {
//             tenant_id: Some(request.tenant_id.into()),
//             lens_uid: Some(request.lens_uid.into()),
//         }
//     }
// }
//
//
// impl type_url::TypeUrl for CloseLensRequest{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensRequest";
// }
//
// impl serde_impl::ProtobufSerializable for CloseLensRequest {
//     type ProtobufMessage = CloseLensRequestProto;
// }
//
// //
// // CloseLensResponse
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct CloseLensResponse {
//
// }
//
// impl TryFrom<CloseLensResponseProto> for CloseLensResponse {
//     type Error = SerDeError;
//     fn try_from(response_proto: CloseLensResponseProto) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl TryFrom<CloseLensResponse> for CloseLensResponseProto{
//     type Error = SerDeError;
//
//     fn try_from(response: CloseLensResponse) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl type_url::TypeUrl for CloseLensResponse{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensResponse";
// }
//
// impl serde_impl::ProtobufSerializable for CloseLensResponse {
//     type ProtobufMessage = CloseLensResponseProto;
// }
//
// //
// // AddNodeToScopeRequest
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct AddNodeToScopeRequest {
//     pub tenant_id: Uuid,
//     pub lens_uid: u64,
//     pub uid: u64,
// }
//
// impl TryFrom<AddNodeToScopeRequestProto> for AddNodeToScopeRequest {
//     type Error = SerDeError;
//
//     fn try_from(request_proto: AddNodeToScopeRequestProto) -> Result<Self, Self::Error> {
//         let tenant_id = request_proto
//             .tenant_id
//             .ok_or(SerDeError::MissingField("tenant_id"))?;
//
//         let lens_uid = request_proto
//             .lens_uid
//             .ok_or(SerDeError::MissingField("lens_uid"))?;
//
//         let uid = request_proto
//             .uid
//             .ok_or(SerDeError::MissingField("uid"))?;
//
//         Ok(AddNodeToScopeRequest {
//             tenant_id: tenant_id.into(),
//             lens_uid: lens_uid.into(),
//             uid: uid.into(),
//         })
//     }
// }
//
// impl From<AddNodeToScopeRequest> for AddNodeToScopeRequestProto {
//     fn from(request: AddNodeToScopeRequest) -> Self {
//         AddNodeToScopeRequestProto{
//             tenant_id: Some(request.tenant_id.into()),
//             lens_uid: Some(request.lens_uid.into()),
//             uid: Some(request.uid.into()),
//         }
//     }
// }
//
// impl type_url::TypeUrl for AddNodeToScopeRequest{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.AddNodeToScopeRequest";
// }
//
// impl serde_impl::ProtobufSerializable for AddNodeToScopeRequest {
//     type ProtobufMessage = AddNodeToScopeRequestProto;
// }
//
// //
// // AddNodeToScopeResponse
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct AddNodeToScopeResponse {
//
// }
//
// impl TryFrom<AddNodeToScopeResponseProto> for AddNodeToScopeResponse {
//     type Error = SerDeError;
//     fn try_from(response_proto: AddNodeToScopeResponseProto) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl TryFrom<AddNodeToScopeResponse> for AddNodeToScopeResponseProto{
//     type Error = SerDeError;
//
//     fn try_from(response: AddNodeToScopeResponse) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl type_url::TypeUrl for AddNodeToScopeResponse{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.AddNodeToScopeResponse";
// }
//
// impl serde_impl::ProtobufSerializable for AddNodeToScopeResponse {
//     type ProtobufMessage = AddNodeToScopeResponseProto;
// }
//
//
// //
// // RemoveNodeFromScopeRequest
// //
// #[derive(Debug, Clone, PartialEq)]
// pub struct RemoveNodeFromScopeRequest{
//     pub tenant_id: Uuid,
//     pub lens_uid: u64,
// }
//
// impl TryFrom<RemoveNodeFromScopeRequestProto> for RemoveNodeFromScopeRequest {
//     type Error = SerDeError;
//
//     fn try_from(request_proto: RemoveNodeFromScopeRequestProto) -> Result<Self, Self::Error>{
//         let tenant_id = request_proto
//             .tenant_id
//             .ok_or(SerDeError::MissingField("tenant_id"))?;
//
//
//         Ok(RemoveNodeFromScopeRequest {
//             tenant_id: tenant_id.into(),
//             lens_uid: request_proto.lens_uid.into(),
//         })
//     }
// }
//
// impl From<RemoveNodeFromScopeRequest> for RemoveNodeFromScopeRequestProto{
//     fn from(request: RemoveNodeFromScopeRequest) -> Self{
//         RemoveNodeFromScopeRequestProto{
//             tenant_id: Some(request.tenant_id.into()),
//             lens_uid: Some(request.lens_uid.into()),
//         }
//     }
// }
//
// impl type_url::TypeUrl for RemoveNodeFromScopeRequest{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromScopeRequest";
// }
//
// impl serde_impl::ProtobufSerializable for RemoveNodeFromScopeRequest {
//     type ProtobufMessage = RemoveNodeFromScopeRequestProto;
// }
//
// //
// // RemoveNodeFromScopeResponse
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct RemoveNodeFromScopeResponse {
//
// }
//
// impl TryFrom<RemoveNodeFromScopeResponseProto> for RemoveNodeFromScopeResponse {
//     type Error = SerDeError;
//     fn try_from(response_proto: RemoveNodeFromScopeResponseProto) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl TryFrom<RemoveNodeFromScopeResponse> for RemoveNodeFromScopeResponseProto{
//     type Error = SerDeError;
//
//     fn try_from(response: RemoveNodeFromScopeResponse) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl type_url::TypeUrl for RemoveNodeFromScopeResponse{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromScopeResponse";
// }
//
// impl serde_impl::ProtobufSerializable for RemoveNodeFromScopeResponse {
//     type ProtobufMessage = RemoveNodeFromScopeResponseProto;
// }
//
// //
// // RemoveNodeFromAllScopesRequest
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct RemoveNodeFromAllScopesRequest{
//     pub tenant_id: Uuid, //
//     pub uid: u64, // never 0, Refinement Type
// }
//
// impl TryFrom<RemoveNodeFromAllScopesRequestProto> for RemoveNodeFromAllScopesRequest {
//     type Error = SerDeError;
//
//     fn try_from(request_proto: RemoveNodeFromAllScopesRequest) -> Result<Self, Self::Error> {
//         let tenant_id = request_proto
//             .tenant_id
//             .ok_or(SerDeError::MissingField("tenant_id"))?;
//
//         let uid = request_proto
//             .uid
//             .ok_or(SerDeError::MissingField("uid"))?;
//
//         if uid = 0 {
//
//         } else {
//
//         }
//
//         // check for invalid values for u64 and send back error otherwise deserailize
//         Ok(RemoveNodeFromAllScopesRequest {
//             tenant_id: tenant_id.into(),
//             uid: uid.into(),
//         })
//     }
// }
//
// impl From<RemoveNodeFromAllScopesRequest> for RemoveNodeFromAllScopesRequestProto {
//     fn from(request: RemoveNodeFromAllScopesRequest) -> Self {
//         RemoveNodeFromAllScopesRequestProto {
//             tenant_id: Some(request.tenant_id.into()),
//             uid: Some(request.uid.into()),
//         }
//     }
// }
//
// impl type_url::TypeUrl for RemoveNodeFromAllScopesRequest{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromAllScopesRequest";
// }
//
// impl serde_impl::ProtobufSerializable for RemoveNodeFromAllScopesRequest {
//     type ProtobufMessage = RemoveNodeFromAllScopesRequestProto;
// }
//
// //
// // RemoveNodeFromAllScopesResponse
// //
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct RemoveNodeFromAllScopesResponse {
//
// }
//
// impl TryFrom<RemoveNodeFromAllScopesResponse> for RemoveNodeFromAllScopesResponseProto{
//     type Error = SerDeError;
//
//     fn try_from(response: RemoveNodeFromAllScopesResponse) -> Result<Self, Self::Error> {
//         Ok(Self{})
//     }
// }
//
// impl type_url::TypeUrl for RemoveNodeFromAllScopesResponse{
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.RemoveNodeFromAllScopesResponse";
// }
//
// impl serde_impl::ProtobufSerializable for RemoveNodeFromAllScopesResponse {
//     type ProtobufMessage = RemoveNodeFromAllScopesResponseProto;
// }