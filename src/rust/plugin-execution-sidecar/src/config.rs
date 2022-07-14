#[derive(clap::Parser, Clone, Debug)]
pub struct PluginExecutorConfig {
    #[clap(long, env = "PLUGIN_EXECUTOR_PLUGIN_ID")]
    pub plugin_id: uuid::Uuid,

    #[clap(long, env = "PLUGIN_EXECUTOR_TENANT_ID")]
    pub tenant_id: uuid::Uuid,
}
