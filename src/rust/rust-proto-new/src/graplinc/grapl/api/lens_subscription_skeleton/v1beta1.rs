use bytes::Bytes;

use crate::{
    graplinc::common::v1beta1::{
        SystemTime,
        Uuid,
    },
    protobufs::graplinc::grapl::api::lens_subscription::v1beta1::{
        PublishRawLogRequest as PublishRawLogRequestProto,
        PublishRawLogResponse as PublishRawLogResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

// message LensUpdate {
// // Type of Operation
// string Operation = 1;
// //  Operation operation = 1; // This needs to change to Operation!!!!!
// // Type of Lens
// string lens_type = 2;
// // Name of Lens
// string lens_name = 3;
// // Id of Tenant;Lens should be updated for
// graplinc.common.v1beta1.Uuid tenant_id = 4;
// }


///
/// Lens Update
///

#[derive(Debug, Clone, PartialEq)]
pub struct LensUpdate {
    pub operation: string, // needs to be Operation TypeUrl
    pub lens_type: string,
    pub lens_name: string,
    pub tenant_id: Uuid,
}


