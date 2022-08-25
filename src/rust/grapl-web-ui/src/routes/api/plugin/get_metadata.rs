use std::sync::Mutex;

use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetPluginRequest,
    PluginRegistryServiceClient,
};
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

#[tracing::instrument(skip(plugin_registry_client, user, data), fields(
    username = tracing::field::Empty
))]
pub(super) async fn get_metadata(
    plugin_registry_client: web::Data<Mutex<PluginRegistryServiceClient>>,
    user: crate::authn::AuthenticatedUser,
    data: web::Query<GetPluginMetadataParameters>,
) -> Result<impl Responder, PluginError> {
    let tenant_id = user.get_organization_id().to_owned();

    let request = GetPluginRequest::new(data.plugin_id, tenant_id);

    let mut plugin_registry_client = plugin_registry_client.lock().unwrap();
    let plugin_registry_response = plugin_registry_client.get_plugin(request).await?;

    let plugin_metadata = plugin_registry_response.plugin_metadata();

    let web_response = GetPluginMetadataResponse {
        plugin_id: plugin_registry_response.plugin_id(),
        display_name: plugin_metadata.display_name().to_string(),
        plugin_type: plugin_metadata.plugin_type().type_name().to_string(),
        event_source_id: plugin_metadata.event_source_id(),
    };

    Ok(HttpResponse::Ok().json(web_response))
}
