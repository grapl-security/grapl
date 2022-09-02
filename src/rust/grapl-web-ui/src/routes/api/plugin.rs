pub mod create;
pub mod deploy;
mod error;
pub mod get_analyzers;
pub mod get_deployment;
pub mod get_health;
pub mod get_metadata;
pub mod tear_down;

use actix_web::web;
pub use error::PluginError;
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetPluginRequest,
    PluginMetadata,
    PluginRegistryServiceClient,
};

pub(super) fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/create", web::post().to(create::create));
    cfg.route("/deploy", web::post().to(deploy::deploy));
    cfg.route(
        "/get_deployment",
        web::get().to(get_deployment::get_deployment),
    );
    cfg.route(
        "/get_analyzers",
        web::get().to(get_analyzers::get_analyzers),
    );
    cfg.route("/get_health", web::get().to(get_health::get_health));
    cfg.route("/get_metadata", web::get().to(get_metadata::get_metadata));
    cfg.route("/tear_down", web::post().to(tear_down::tear_down));
}

/// Validate the authenticated user has permissions to operate on the supplied plugin ID.
///
/// Returns validated Plugin metadata.
#[tracing::instrument(skip(client))]
async fn verify_plugin_ownership<'a>(
    client: &mut PluginRegistryServiceClient,
    user: &crate::authn::AuthenticatedUser,
    plugin_id: uuid::Uuid,
) -> Result<PluginMetadata, PluginError> {
    let authenticated_tenant_id = user.get_organization_id().to_owned();
    let get_plugin_request = GetPluginRequest::new(plugin_id, authenticated_tenant_id);

    tracing::debug!(message = "getting plugin metadata", ?get_plugin_request);

    let get_plugin_response = client
        .get_plugin(get_plugin_request)
        .timeout(std::time::Duration::from_secs(5))
        .await??;
    let plugin_metadata = get_plugin_response.plugin_metadata().to_owned();

    if authenticated_tenant_id != plugin_metadata.tenant_id() {
        tracing::warn!(
            message = "user requested to deploy plugin owned by another tenant",
            ?user
        );

        return Err(PluginError::Unauthorized);
    }

    Ok(plugin_metadata)
}
