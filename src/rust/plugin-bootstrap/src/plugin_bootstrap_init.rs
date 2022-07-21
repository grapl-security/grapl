use std::os::unix::fs::PermissionsExt;

use clap::Parser;
use rust_proto::graplinc::grapl::api::plugin_bootstrap::v1beta1::{
    GetBootstrapRequest,
    GetBootstrapResponse,
};
use rust_proto_clients::{
    get_grpc_client,
    PluginBootstrapClientConfig,
};

static PLUGIN_BINARY_PATH: &str = "/usr/local/bin/grapl-plugin";
static CLIENT_CERTIFICATE_PATH: &str = "/etc/ssl/private/plugin-client-cert.pem";
static PLUGIN_CONFIG_PATH: &str = "/etc/systemd/system/plugin.service.d/override.conf";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    let client_config = PluginBootstrapClientConfig::parse();
    let mut bootstrap_client = get_grpc_client(client_config).await?;

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
