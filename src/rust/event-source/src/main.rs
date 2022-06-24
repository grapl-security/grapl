use clap::Parser;
use event_source::{
    config::EventSourceConfig,
    server::exec_service,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();
    let config = EventSourceConfig::parse();
    exec_service(config).await?;
    Ok(())
}
