use std::time::Duration;

use rust_proto::graplinc::grapl::api::plugin_registry::v1beta1::{
    GetPluginHealthRequest,
    PluginHealthStatus,
    PluginRegistryClient,
};

/// After a deploy, plugins seem to take quite some time to deploy.
/// This function lets us assert that a plugin eventually reaches
/// a certain state within <some duration>.
pub async fn assert_eventual_health(
    client: &PluginRegistryClient,
    plugin_id: uuid::Uuid,
    expected: PluginHealthStatus,
    timeout: Duration,
) -> eyre::Result<()> {
    let mut client = client.clone();
    let start_time = std::time::SystemTime::now();
    let sleep_between_tries = Duration::from_secs(3);

    loop {
        let elapsed = start_time.elapsed()?;
        if elapsed > timeout {
            eyre::bail!("Plugin {plugin_id} never reached {expected:?} within {timeout:?}");
        }

        let actual_health_status = client
            .get_plugin_health(GetPluginHealthRequest::new(plugin_id))
            .await?
            .health_status();
        if expected == actual_health_status {
            tracing::debug!(
                "Plugin {plugin_id} reached expected status {expected:?} after {elapsed:?}"
            );
            return Ok(());
        } else {
            tokio::time::sleep(sleep_between_tries).await;
        }
    }
}
