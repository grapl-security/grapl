use actix_web::{
    web,
    HttpResponse,
};
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    PluginRegistryServiceClient,
    TearDownPluginRequest,
};

use super::PluginError;

#[derive(serde::Deserialize)]
pub(super) struct DeployPluginParameters {
    plugin_id: uuid::Uuid,
}

#[tracing::instrument(skip(plugin_registry_client, data))]
pub(super) async fn tear_down(
    plugin_registry_client: web::Data<PluginRegistryServiceClient>,
    user: crate::authn::AuthenticatedUser,
    data: web::Json<DeployPluginParameters>,
) -> Result<impl actix_web::Responder, PluginError> {
    let requested_plugin_id = data.plugin_id;

    let mut plugin_registry_client = plugin_registry_client.get_ref().clone();

    super::verify_plugin_ownership(&mut plugin_registry_client, &user, requested_plugin_id).await?;

    let request = TearDownPluginRequest::new(requested_plugin_id);

    tracing::debug!(message = "tearing down plugin", ?request);

    let response = plugin_registry_client
        .tear_down_plugin(request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    tracing::debug!(?response);

    Ok(HttpResponse::Ok().finish())
}
