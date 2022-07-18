use generator_sdk::server::{
    self,
    GeneratorServiceConfig,
};
use grapl_tracing::setup_tracing;
use sysmon_generator::api;

const SERVICE_NAME: &'static str = "sysmon-generator";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME);

    let config = GeneratorServiceConfig::from_env_vars();
    let generator = api::SysmonGenerator {};
    server::exec_service(generator, config).await
}
