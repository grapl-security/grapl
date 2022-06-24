use std::{
    os::unix::fs::PermissionsExt,
    time::Duration,
};

use rust_proto_new::{
    graplinc::grapl::api::plugin_bootstrap::v1beta1::{
        client::PluginBootstrapClient,
        GetBootstrapRequest,
        GetBootstrapResponse,
    },
    protocol::healthcheck::client::HealthcheckClient,
};

static PLUGIN_BINARY_PATH: &str = "/usr/local/bin/grapl-plugin";
static CLIENT_CERTIFICATE_PATH: &str = "/etc/ssl/private/plugin-client-cert.pem";
static PLUGIN_CONFIG_PATH: &str = "/etc/systemd/system/plugin.service.d/override.conf";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    let endpoint = std::env::var("PLUGIN_BOOTSTRAP_CLIENT_ADDRESS")?;
    let plugin_bootstrap_polling_interval_ms: u64 =
        std::env::var("PLUGIN_BOOTSTRAP_POLLING_INTERVAL_MS")?.parse()?;

    tracing::info!(
        message = "waiting 5s for plugin-bootstrap to report healthy",
        endpoint = %endpoint,
    );

    HealthcheckClient::wait_until_healthy(
        endpoint.clone(),
        "graplinc.grapl.api.plugin_bootstrap.v1beta1.PluginBootstrapService",
        Duration::from_secs(5), // TODO: parametrize
        Duration::from_millis(plugin_bootstrap_polling_interval_ms),
    )
    .await?;

    let mut bootstrap_client = PluginBootstrapClient::connect(endpoint.clone()).await?;

    let GetBootstrapResponse {
        plugin_payload,
        client_certificate,
    } = bootstrap_client
        .get_bootstrap(GetBootstrapRequest {})
        .await?;

    std::fs::write(PLUGIN_BINARY_PATH, plugin_payload.plugin_binary)?;
    std::fs::write(
        CLIENT_CERTIFICATE_PATH,
        client_certificate.client_certificate,
    )?;

    std::fs::set_permissions(PLUGIN_BINARY_PATH, std::fs::Permissions::from_mode(0o755))?;
    std::fs::set_permissions(
        CLIENT_CERTIFICATE_PATH,
        std::fs::Permissions::from_mode(0o400),
    )?;

    let config_file = std::fs::File::create(PLUGIN_CONFIG_PATH)?;
    config_file.set_permissions(std::fs::Permissions::from_mode(0o655))?;

    Ok(())
}
