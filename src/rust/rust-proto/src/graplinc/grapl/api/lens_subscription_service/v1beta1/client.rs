#![allow(warnings)]

use futures::Stream;
use crate::graplinc::grapl::api::lens_subscription_service::v1beta1::messages::{SubscribeToLensRequest, SubscribeToLensResponse};
use crate::protobufs::graplinc::grapl::api::lens_subscription_service::v1beta1::lens_subscription_service_client::LensSubscriptionServiceClient;
use crate::protobufs::graplinc::grapl::api::lens_subscription_service::v1beta1::{
    SubscribeToLensRequest as SubscribeToLensRequestProto,
};
use crate::protocol::status::Status;
use crate::SerDeError;
use tokio_stream::StreamExt;

#[derive(thiserror::Error, Debug)]
pub enum LensSubscriptionClientError {
    #[error("Failed to deserialize response {0}")]
    SerDeError(#[from] SerDeError),
    #[error("Status {0}")]
    Status(#[from] Status),
    #[error("ConnectError")]
    ConnectError(tonic::transport::Error),
}

#[derive(Clone)]
pub struct LensSubscriptionClient {
    inner: LensSubscriptionServiceClient<tonic::transport::Channel>,
}

impl LensSubscriptionClient {
    pub async fn connect<T>(endpoint: T) -> Result<Self, LensSubscriptionClientError>
        where
            T: TryInto<tonic::transport::Endpoint>,
            T::Error: std::error::Error + Send + Sync + 'static,
    {
        Ok(LensSubscriptionClient {
            inner: LensSubscriptionServiceClient::connect(endpoint)
                .await
                .map_err(LensSubscriptionClientError::ConnectError)?,
        })
    }

    pub async fn subscribe_to_lens(&mut self, request: SubscribeToLensRequest) -> Result<impl Stream<Item=Result<SubscribeToLensResponse,LensSubscriptionClientError>>, LensSubscriptionClientError> {
        let request: SubscribeToLensRequestProto = request.into();
        let response = self.inner.subscribe_to_lens(
            tonic::Request::new(request)
        ).await
            .map_err(Status::from)?;
        let response = response.into_inner();
        let response = StreamExt::map(response, |response| {
            match response {
                Ok(update) => Ok(update.try_into()?),
                Err(e) => Err(LensSubscriptionClientError::Status(Status::from(e))),
            }
        });

        Ok(response)
    }
}
