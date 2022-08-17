use clap::Parser;
use rust_proto::graplinc::grapl::api::schema_manager::v1beta1::server::SchemaManagerServiceServer;
use schema_manager::{
    config::SchemaServiceConfig,
    server::SchemaManager,
};

const SERVICE_NAME: &'static str = "schema-manager";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = grapl_tracing::setup_tracing(SERVICE_NAME)?;
    let config = SchemaServiceConfig::parse();
    let pool = sqlx::PgPool::connect(&config.schema_db_config.to_postgres_url()).await?;

    let schema_manager_service = SchemaManager { pool };

    let (_tx, rx) = tokio::sync::oneshot::channel();
    SchemaManagerServiceServer::builder(
        schema_manager_service,
        config.schema_service_bind_address,
        rx,
    )
    .build()
    .serve()
    .await?;

    Ok(())
}
