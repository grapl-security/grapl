mod error;
mod response;

use actix_web::web;
use error::WebResponseError;
use futures_util::{
    StreamExt,
    TryStreamExt,
};
use response::WebResponse;
use rust_proto::graplinc::grapl::{
    api::lens_subscription_service::v1beta1::messages::SubscribeToLensRequest,
    common::v1beta1::types::Uid,
};

#[derive(serde::Deserialize)]
pub(super) struct SubscribeParameters {
    lens_id: u64,
}

#[tracing::instrument(skip(lens_client_url, data, user), fields(
    username = tracing::field::Empty
))]
pub(super) async fn subscribe(
    lens_client_url: web::Data<crate::LensSubscriptionUrl>,
    user: crate::authn::AuthenticatedUser,
    data: web::Json<SubscribeParameters>,
) -> Result<impl actix_web::Responder, WebResponseError> {
    tracing::debug!(message = "subscribing to lens", username = user.get_username(), lens_id =? data.lens_id);

    let lens_uid = Uid::from_u64(data.lens_id).ok_or(WebResponseError::LensUidZero)?;
    let tenant_id = uuid::Uuid::parse_str(user.get_organization_id()).map_err(|source| {
        WebResponseError::InavlidTenantID {
            tenant_id: user.get_organization_id().to_string(),
            source,
        }
    })?;

    let req = SubscribeToLensRequest {
        tenant_id,
        lens_uid,
    };

    let lens_client_url = lens_client_url.into_inner().get_ref().to_string();
    let mut lens_client = crate::LensSubscriptionClient::connect(lens_client_url).await?;

    let stream = lens_client.subscribe_to_lens(req).await?;

    let bytes = stream
        .map_ok(WebResponse::from)
        .map(|response| match response {
            Ok(ref response) => response.as_json_bytes(),
            Err(e) => Err(e.into()),
        });

    Ok(actix_web::HttpResponse::Ok().streaming(bytes))
}
