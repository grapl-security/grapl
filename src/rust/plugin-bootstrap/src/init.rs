use std::os::unix::fs::PermissionsExt;

use plugin_bootstrap::client::PluginBootstrapClient;
use rust_proto::plugin_bootstrap::{
    GetBootstrapInfoRequest,
    GetBootstrapInfoResponse,
};

static PLUGIN_BINARY_PATH: &str = "/usr/local/bin/grapl-plugin";
static CLIENT_CERTIFICATE_PATH: &str = "/etc/ssl/private/plugin-client-cert.pem";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    std::fs::set_permissions(PLUGIN_BINARY_PATH, std::fs::Permissions::from_mode(0o400))?;

    Ok(())
}
