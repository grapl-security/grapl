use actix_web::{
    web,
    HttpResponse,
};
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    DeployPluginRequest,
    PluginRegistryClient,
};

use super::PluginError;

#[derive(serde::Deserialize)]
pub(super) struct DeployPluginParameters {
    plugin_id: uuid::Uuid,
}

#[tracing::instrument(skip(plugin_registry_client, data))]
pub(super) async fn deploy(
    plugin_registry_client: web::Data<PluginRegistryClient>,
    user: crate::authn::AuthenticatedUser,
    data: web::Json<DeployPluginParameters>,
) -> Result<impl actix_web::Responder, PluginError> {
    let requested_plugin_id = data.plugin_id;

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();

    super::verify_plugin_ownership(&mut plugin_registry_client, &user, requested_plugin_id).await?;

    let request = DeployPluginRequest::new(requested_plugin_id);

    tracing::debug!(message = "deploying plugin", ?request);

    let response = plugin_registry_client
        .deploy_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    tracing::debug!(?response);

    Ok(HttpResponse::Ok().finish())
}
