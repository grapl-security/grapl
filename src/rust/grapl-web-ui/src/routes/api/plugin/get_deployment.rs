use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetPluginDeploymentRequest,
    PluginDeploymentStatus,
    PluginRegistryServiceClient,
};

use super::PluginError;

#[derive(serde::Deserialize)]
pub(super) struct GetDeploymentParameters {
    plugin_id: uuid::Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PluginDeploymentResponse {
    pub plugin_id: uuid::Uuid,
    pub timestamp: std::time::SystemTime,
    pub status: String,
    pub deployed: bool,
}

#[tracing::instrument(skip(plugin_registry_client, data))]
pub(super) async fn get_deployment(
    plugin_registry_client: web::Data<PluginRegistryServiceClient>,
    user: crate::authn::AuthenticatedUser,
    data: web::Query<GetDeploymentParameters>,
) -> Result<impl Responder, PluginError> {
    let requested_plugin_id = data.plugin_id;

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();

    super::verify_plugin_ownership(&mut plugin_registry_client, &user, requested_plugin_id).await?;

    let request = GetPluginDeploymentRequest::new(requested_plugin_id);
    let response = plugin_registry_client
        .get_plugin_deployment(request)
        .await?
        .plugin_deployment();

    //TODO: It'd be great if rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginDeploymentStatus
    // implemented Display, or serde::Serialize
    let status = match response.status() {
        PluginDeploymentStatus::Fail => "fail",
        PluginDeploymentStatus::Success => "success",
        PluginDeploymentStatus::Unspecified => "unspecified",
    };

    let web_response = PluginDeploymentResponse {
        plugin_id: response.plugin_id(),
        timestamp: response.timestamp(),
        status: status.to_string(),
        deployed: response.deployed(),
    };

    Ok(HttpResponse::Ok().json(web_response))
}
