use tracing::info;
use clap::Parser;
use grapl_tracing::setup_tracing;
use db_schema_manager::server::{
    exec_service,
};
use db_schema_manager::config::DbSchemaManagerServiceConfig;

const SERVICE_NAME: &'static str = "db-schema-manager";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME)?;
    let config = DbSchemaManagerServiceConfig::parse();
    info!(message="Starting Db Schema Manager Service", config=?config);

    exec_service(config).await?;
    Ok(())
}


