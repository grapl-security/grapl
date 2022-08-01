use clap::Parser;
use event_source::{
    config::EventSourceConfig,
    server::exec_service,
};
use grapl_tracing::setup_tracing;

const SERVICE_NAME: &'static str = "event-source";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let config = EventSourceConfig::parse();
    exec_service(config).await?;
    Ok(())
}
