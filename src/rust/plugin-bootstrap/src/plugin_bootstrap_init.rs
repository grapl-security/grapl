use std::os::unix::fs::PermissionsExt;

use figment::{
    providers::Env,
    Figment,
};
use grapl_tracing::setup_tracing;
use rust_proto::graplinc::grapl::api::{
    client::Connect,
    plugin_bootstrap::v1beta1::{
        client::PluginBootstrapClient,
        GetBootstrapRequest,
        GetBootstrapResponse,
    },
};

static PLUGIN_BINARY_PATH: &str = "/usr/local/bin/grapl-plugin";
static CLIENT_CERTIFICATE_PATH: &str = "/etc/ssl/private/plugin-client-cert.pem";
static PLUGIN_CONFIG_PATH: &str = "/etc/systemd/system/plugin.service.d/override.conf";
const SERVICE_NAME: &'static str = "plugin-bootstrap-init";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;

    let client_config = Figment::new()
        .merge(Env::prefixed("PLUGIN_BOOTSTRAP_CLIENT_"))
        .extract()?;
    let mut bootstrap_client = PluginBootstrapClient::connect(client_config).await?;

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
