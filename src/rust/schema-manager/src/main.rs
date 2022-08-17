use std::time::Duration;

use clap::Parser;
use grapl_config::PostgresClient;
use rust_proto::{
    graplinc::grapl::api::schema_manager::v1beta1::server::SchemaManagerServer,
    protocol::healthcheck::HealthcheckStatus,
};
use schema_manager::{
    config::SchemaManagerConfig,
    db::client::SchemaDbClient,
    server::SchemaManager,
};
use tokio::net::TcpListener;

const SERVICE_NAME: &'static str = "schema-manager";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = grapl_tracing::setup_tracing(SERVICE_NAME)?;
    let config = SchemaManagerConfig::parse();

    let db_client = SchemaDbClient::init_with_config(config.schema_db_config.clone()).await?;
    let schema_manager_api_impl = SchemaManager { db_client };

    exec_service(config, schema_manager_api_impl).await
}

pub async fn exec_service(
    config: SchemaManagerConfig,
    api_impl: SchemaManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let healthcheck_polling_interval_ms = config.schema_manager_healthcheck_polling_interval_ms;

    let (server, _shutdown_tx) = SchemaManagerServer::new(
        api_impl,
        TcpListener::bind(config.schema_manager_bind_address.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );
    tracing::info!(message = "starting gRPC server",);

    Ok(server.serve().await?)
}
