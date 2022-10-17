use actix_web::{
    web,
    HttpResponse,
};
use futures::StreamExt;
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::pipeline_ingress::v1beta1::{
    client::PipelineIngressClient,
    PublishRawLogRequest,
};

use super::IngressError;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct PublishLogResponse {
    pub created_time: std::time::SystemTime,
}

#[tracing::instrument(skip(plugin_registry_client, body))]
pub(super) async fn publish(
    plugin_registry_client: web::Data<PipelineIngressClient>,
    user: crate::authn::AuthenticatedUser,
    path: web::Path<uuid::Uuid>,
    mut body: web::Payload,
) -> Result<impl actix_web::Responder, IngressError> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        let item = item?;
        bytes.extend_from_slice(&item);
    }

    let authenticated_tenant_id = user.get_organization_id().to_owned();
    let event_source_id = path.into_inner();
    let publish_log_request = PublishRawLogRequest::new(
        event_source_id,
        authenticated_tenant_id,
        bytes::Bytes::from(bytes),
    );

    tracing::debug!(message = "publish log", ?publish_log_request);

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();
    let response = plugin_registry_client
        .publish_raw_log(publish_log_request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let created_time = response.created_time();

    Ok(HttpResponse::Ok().json(PublishLogResponse { created_time }))
}
