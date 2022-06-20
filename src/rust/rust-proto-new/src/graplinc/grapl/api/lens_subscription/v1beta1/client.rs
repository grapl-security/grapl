use std::fmt::Debug;

use futures::{
    Stream,
    StreamExt,
};
use grapl_utils::iter_ext::GraplIterExt;
use proto::lens_subscription_service_client::LensSubscriptionServiceClient as LensSubscriptionServiceClientProto;

use crate::{
    graplinc::grapl::api::lens_subscription::v1beta1 as native,
    protobufs::graplinc::grapl::api::lens_subscription::v1beta1 as proto,
    graplinc::grapl::api::lens_subscription::v1beta1::LensSubscriptionProto,
    graplinc::grapl::api::lens_subscription::v1beta1::{SubscribeToLensRequest, SubscribeToLensResponse},
    SerDeError,
};


#[derive(Debug, thiserror::Error)]
pub enum LensSubscriptionServiceClientError {
    #[error("TransportError {0}")]
    TransportError(#[from] tonic::transport::Error),
    #[error("ErrorStatus {0}")]
    ErrorStatus(#[from] tonic::Status),
    #[error("LensSubscriptionDeserializationError {0}")]
    LensSubscriptionDeserializationError(#[from] SerDeError),
}

#[derive(Clone)]
pub struct LensSubscriptionServiceClient {
    proto_client: LensSubscriptionServiceClientProto<tonic::transport::Channel>,
}

impl LensSubscriptionServiceClient {
    #[tracing::instrument(err)]
    pub async fn connect<T>(endpoint: T) -> Result<Self, LensSubscriptionServiceClientError>
        where
            T: std::convert::TryInto<tonic::transport::Endpoint> + Debug,
            T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(LensSubscriptionServiceClient {
            proto_client: LensSubscriptionServiceClientProto::connect(endpoint).await?,
        })
    }

    pub async fn lens_subscription<S>(
        &mut self,
        request: S,
    ) -> Result<SubscribeToLensResponse, LensSubscriptionServiceClientError>
        where
            S: Stream<Item = SubscribeToLensRequest> + Send + 'static,
    {
        let response = self
            .proto_client
            .lens_subscription(request.map(SubscribeToLensRequest::from))
            .await?;

        let response = SubscribeToLensResponse::try_from(response.into_inner())?;

        Ok(response)
    }

    pub async fn subscribe_to_lens(
        &mut self,
        operation: impl Sized + Iterator<Item = u8> + Send + 'static,
    ) -> Result<LensSubscriptionProto, LensSubscriptionServiceClientError> {
        let request = futures::stream::iter(std::iter::once(
            SubscribeToLensRequest, // not sure if this is correct
        ))
            .chain(futures::stream::iter(
                operation.map(SubscribeToLensRequest),
            ));
        self.subscribe_to_lens(request).await
    }
}