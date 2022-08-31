use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetPluginHealthRequest,
    PluginHealthStatus,
    PluginRegistryServiceClient,
};
use uuid::Uuid;

use super::PluginError;

#[derive(serde::Deserialize)]
pub(super) struct GetPluginHealthParameters {
    plugin_id: Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GetPluginHealthResponse {
    pub health_status: String,
}

#[tracing::instrument(skip(plugin_registry_client, data))]
pub(super) async fn get_health(
    plugin_registry_client: web::Data<PluginRegistryServiceClient>,
    user: crate::authn::AuthenticatedUser,
    data: web::Query<GetPluginHealthParameters>,
) -> Result<impl Responder, PluginError> {
    let requested_plugin_id = data.plugin_id;

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();

    super::verify_plugin_ownership(&mut plugin_registry_client, &user, requested_plugin_id).await?;

    let request = GetPluginHealthRequest::new(data.plugin_id);

    tracing::debug!(message = "getting plugin metadata", ?request);

    let plugin_registry_response = plugin_registry_client.get_plugin_health(request).await?;

    //TODO: It'd be great if rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginHealthStatus
    // implemented Display, or serde::Serialize
    let health_status = match plugin_registry_response.health_status() {
        PluginHealthStatus::Dead => "dead",
        PluginHealthStatus::NotDeployed => "not deployed",
        PluginHealthStatus::Pending => "pending",
        PluginHealthStatus::Running => "running",
    };

    let web_response = GetPluginHealthResponse {
        health_status: health_status.to_string(),
    };

    Ok(HttpResponse::Ok().json(web_response))
}
