use rust_proto_new::graplinc::grapl::api::schema_manager::v1beta1::server::SchemaManagerServiceServer;
use schema_manager::{
    config::SchemaServiceConfig,
    server::SchemaManager,
};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SchemaServiceConfig::from_args();
    let pool = sqlx::PgPool::connect(&config.schema_db_config.to_postgres_url()).await?;

    let schema_manager_service = SchemaManager { pool };

    let (_tx, rx) = tokio::sync::oneshot::channel();
    SchemaManagerServiceServer::builder(
        schema_manager_service,
        config.schema_servicebind_address,
        rx,
    )
    .build()
    .serve()
    .await?;

    Ok(())
}
