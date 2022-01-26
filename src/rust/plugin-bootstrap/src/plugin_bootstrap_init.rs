use std::os::unix::fs::PermissionsExt;

use plugin_bootstrap::client::PluginBootstrapClient;
use rust_proto::plugin_bootstrap::{
    GetBootstrapInfoRequest,
    GetBootstrapInfoResponse,
};

static PLUGIN_BINARY_PATH: &str = "/usr/local/bin/grapl-plugin";
static CLIENT_CERTIFICATE_PATH: &str = "/etc/ssl/private/plugin-client-cert.pem";
static PLUGIN_CONFIG_PATH: &str = "/etc/systemd/system/plugin.service.d/override.conf";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    let mut bootstrap_client = PluginBootstrapClient::from_env().await?;
    let GetBootstrapInfoResponse {
        plugin_payload,
        client_certificate,
    } = bootstrap_client
        .get_bootstrap_info(GetBootstrapInfoRequest {})
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
    config_file.set_permissions(std::fs::Permissions::from_mode(0o755))?;

    Ok(())
}
