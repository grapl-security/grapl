use bytes::Bytes;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::graplinc::grapl::api::lens_subscription::v1beta1::{
        LensUpdate as LensUpdateProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

///
/// Lens Update
///
#[derive(Debug, Clone, PartialEq)]
pub struct LensUpdate {
    pub operation: string,
    // needs to be Operation TypeUrl
    pub lens_type: string,
    pub lens_name: string,
    pub tenant_id: Uuid,
}

impl TryFrom<LensUpdateProto> for LensUpdate {
    type Error = SerDeError;

    fn try_from(lens_update: LensUpdateProto) -> Result<Self, Self::Error> {
        let operation = lens_update.operation;
        let lens_type = lens_update.lens_type;
        let lens_name = lens_update.lens_name;
        let tenant_id = lens_update.tenant_id.ok_or(SerDeError::MissingField("tenant_id"))?;
        Ok(LensUpdate {
            operation,
            lens_type,
            lens_name,
            tenant_id.into(),
        })
    }
}

impl From<LensUpdate> for LensUpdateProto {
    fn from(lens_update: LensUpdate) {
        LensUpdateProto {
            operation: lens_update.operation,
            lens_type: lens_update.lens_type,
            lens_name: lens_update.lens_name,
            tenant_id: Some(request.tenant_id.into()),
        }
    }
}


impl type_url::TypeUrl for LensUpdate {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_subscription.v1beta1.LensUpdate";
}

impl serde_impl::ProtobufSerializable for LensUpdate {
    type ProtobufMessage = LensUpdatetProto;
}
daf