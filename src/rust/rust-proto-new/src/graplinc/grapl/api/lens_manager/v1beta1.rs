use bytes::Bytes;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid
    },
    protobufs::graplinc::grapl::api::lens_manager::v1beta1::{
        CreateLensRequest as CreateLensRequestProto,
        CreateLensResponse as CreateLensResponseProto,
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
    pub is_valid: Boolean,
}

impl TryFrom(CreateLensRequestProto) for CreateLensRequest {
    type Error = SerDeError;

    fn try_from(request_proto: CreateLensRequestProto) -> Result<Self, Self::Error> {
        let tenant_id = request_proto
            .tenant_id
            .ok_or(SerDeError::MissingField("tenant_id"));

        let key = request_proto
            .key
            .ok_or(SerDeError::MissingField("key"));

        let value = request_proto
            .value
            .ok_or(SerDeError::MissingField("value"));

        let is_valid = request_proto
            .is_valid
            .ok_or(SerDeError::MissingField("is_valid"));

        Ok(CreateLensRequest{
            tenant_id: tenant_id.into(),
            key: key.into(),
            value: value.into(),
            is_valid: is_valid.into(),
        })
    }
}

impl From<CreateLensRequest> for CreateLensRequestProto {
    fn from(request: CreateLensRequest) -> Self {
        CreateLensRequestProto {
            tenant_id: Some(request.tenant_id.into()),
            key: Some(request.key.into()),
            value: Some(request.value.into()),
            is_valid: Some(request.value.into()),
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

//
// CreateLensResponse
//

// // Response from creating a lens
// message CreateLensResponse {
// uint64 lens_uid = 2;
// }

#[derive(Debug, Clone, PartialEq)]
pub struct CreateLensResponse {
    pub lens_uid: Uint64, // #TODO: Not sure if this is an actual type
}

impl CreateLensResponse{
    pub fn ok() -> Self {
        CreateLensResponse{
            lens_uid: Uint64,
        }
    }
}

impl TryFrom<CreateLensResponseProto> for CreateLenseResponse {
    type Error = SerDeError;

    fn try_from(response_proto: CreateLensRequestProto) -> Result<Self::Error> {
        let lens_uid = response_proto
            .lens_uid
            .ok_or(SerDeError::MissingField("lens_uid"))?;

        Ok(CreateLensResponse {
            lens_uid: lens_uid.try_into()?,
        })
    }
}

impl TryFrom<CreateLensResponse> for CreateLensResponseProto{
    type Error = SerDeError;

    fn try_from(response: CreateLenseResponse) -> Result<Self, Self::Error>{
        Ok(PublishRawLogResponseProto{
            created_tiem: Some(response.create_time.try_into()?),
        })
    }
}

impl type_url::TypeUrl for CreateLensResponse{
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CreateLensResponse";
}

impl serde_impl::ProtobufSerializable for CreateLensResponse {
    type ProtobufMessage = CreateLensResponseProt; 
}