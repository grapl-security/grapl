
/// A centralized place for constants and common patterns done on the
/// `plugin.nomad` (and hax_docker equivalent)

pub fn job_name() -> &'static str {
    // Matches what's in `plugin.nomad`
    "grapl-plugin"
}

pub fn namespace_name(plugin_id: &uuid::Uuid) -> String {
    format!("plugin-{plugin_id}")
}
