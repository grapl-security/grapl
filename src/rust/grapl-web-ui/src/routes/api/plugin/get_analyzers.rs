use actix_web::{
    web,
    HttpResponse,
    Responder,
};
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetAnalyzersForTenantRequest,
    PluginRegistryClient,
};
use uuid::Uuid;

use super::PluginError;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GetAnalyzersResponse {
    pub plugin_ids: Vec<Uuid>,
}

#[tracing::instrument(skip(plugin_registry_client))]
pub(super) async fn get_analyzers(
    plugin_registry_client: web::Data<PluginRegistryClient>,
    user: crate::authn::AuthenticatedUser,
) -> Result<impl Responder, PluginError> {
    let authenticated_tenant_id = user.get_organization_id();

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();

    let request = GetAnalyzersForTenantRequest::new(authenticated_tenant_id.to_owned());

    let response = plugin_registry_client
        .get_analyzers_for_tenant(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let web_response = GetAnalyzersResponse {
        plugin_ids: response.plugin_ids().to_vec(),
    };

    Ok(HttpResponse::Ok().json(web_response))
}
