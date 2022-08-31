use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::PluginRegistryServiceClient;
use uuid::Uuid;

use super::PluginError;

#[derive(serde::Deserialize)]
pub(super) struct GetPluginMetadataParameters {
    plugin_id: Uuid,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GetPluginMetadataResponse {
    pub plugin_id: Uuid,
    pub display_name: String,
    pub plugin_type: String,
    pub event_source_id: Option<uuid::Uuid>,
}

#[tracing::instrument(skip(plugin_registry_client, data))]
pub(super) async fn get_metadata(
    plugin_registry_client: web::Data<PluginRegistryServiceClient>,
    user: crate::authn::AuthenticatedUser,
    data: web::Query<GetPluginMetadataParameters>,
) -> Result<impl Responder, PluginError> {
    let requested_plugin_id = data.plugin_id;

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();

    let plugin_metadata =
        super::verify_plugin_ownership(&mut plugin_registry_client, &user, requested_plugin_id)
            .await?;

    let web_response = GetPluginMetadataResponse {
        plugin_id: requested_plugin_id,
        display_name: plugin_metadata.display_name().to_string(),
        plugin_type: plugin_metadata.plugin_type().type_name().to_string(),
        event_source_id: plugin_metadata.event_source_id(),
    };

    Ok(HttpResponse::Ok().json(web_response))
}
