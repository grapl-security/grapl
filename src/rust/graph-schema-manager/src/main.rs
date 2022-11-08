use std::time::Duration;

use clap::Parser;
use graph_schema_manager::{
    config::GraphSchemaManagerConfig,
    db::client::SchemaDbClient,
    server::GraphSchemaManager,
};
use grapl_config::PostgresClient;
use rust_proto::graplinc::grapl::api::{
    graph_schema_manager::v1beta1::server::GraphSchemaManagerServer,
    protocol::healthcheck::HealthcheckStatus,
};
use tokio::net::TcpListener;

const SERVICE_NAME: &'static str = "graph-schema-manager";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = grapl_tracing::setup_tracing(SERVICE_NAME)?;
    let config = GraphSchemaManagerConfig::parse();

    let db_client = SchemaDbClient::init_with_config(config.schema_db_config.clone()).await?;
    let graph_schema_manager_api_impl = GraphSchemaManager { db_client };

    exec_service(config, graph_schema_manager_api_impl).await
}

pub async fn exec_service(
    config: GraphSchemaManagerConfig,
    api_impl: GraphSchemaManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let healthcheck_polling_interval_ms =
        config.graph_schema_manager_healthcheck_polling_interval_ms;

    let (server, _shutdown_tx) = GraphSchemaManagerServer::new(
        api_impl,
        TcpListener::bind(config.graph_schema_manager_bind_address.clone()).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );
    tracing::info!(message = "starting gRPC server",);

    Ok(server.serve().await?)
}
