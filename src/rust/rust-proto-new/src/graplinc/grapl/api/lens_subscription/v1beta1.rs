use crate::{
    protobufs::graplinc::grapl::api::lens_subscription::v1beta1::{
        SubscribeToLensRequest as SubscribeToLensRequestProto,

    },
    type_url,
    serde_impl,
};



//
// // Requests a stream of updates for a given lens
// message SubscribeToLensRequest {
// // Lens Subscription Message
// LensSubscription lens_subscription = 1;
// }
//
// message SubscribeToLensResponse {
// // LensUpdate Message
// LensUpdate operation = 1;
// }

#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeToLensRequest{
    pub lens_subscription: LensSubscription, // needs to be lens-subscription
}

impl From<SubscribeToLensRequest> for SubscribeToLensRequestProto {
    fn from(request: SubscribeToLensRequest) -> Self {
        SubscribeToLensRequestProto {
            lens_subscription: request.lens_subscription,

        }
    }
}

impl type_url::TypeUrl for SubscribeToLensRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_subscription.v1beta1.SubscribeToLensRequest";
}

impl serde_impl::ProtobufSerializable for SubscribeToLensRequest {
    type ProtobufMessage = SubscribeToLensRequestProto;
}

//
// CloseLensResponse
//
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct CloseLensResponse {}
//
// impl TryFrom<CloseLensResponseProto> for CloseLensResponse {
//     type Error = SerDeError;
//
//     fn try_from(_response_proto: CloseLensResponseProto) -> Result<Self, Self::Error> {
//         Ok(Self {})
//     }
// }
//
// impl From<CloseLensResponse> for CloseLensResponseProto {
//     fn from(_request: CloseLensResponse) -> Self {
//         CloseLensResponseProto {}
//     }
// }
//
// impl type_url::TypeUrl for CloseLensResponse {
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensResponse";
// }
//
// impl serde_impl::ProtobufSerializable for CloseLensResponse {
//     type ProtobufMessage = CloseLensResponseProto;
// }
