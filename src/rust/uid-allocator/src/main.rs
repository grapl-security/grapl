use clap::Parser;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use rust_proto::graplinc::grapl::api::uid_allocator::v1beta1::server::UidAllocatorServiceServer;
use sqlx::PgPool;
use tracing_subscriber::prelude::*;
use uid_allocator::{
    allocator::UidAllocator,
    config,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());

    // initialize json logging layer
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking);

    // initialize tracing layer
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("pipeline-ingress")
        .install_batch(opentelemetry::runtime::Tokio)?;

    // register a subscriber
    let filter = tracing_subscriber::EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(filter)
        .with(log_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    tracing::info!("logger configured successfully");
    tracing::info!("starting up!");

    let service_config = config::UidAllocatorServiceConfig::parse();
    service_config.validate()?;
    let pool = PgPool::connect(&service_config.counter_db_config.to_postgres_url()).await?;

    sqlx::migrate!().run(&pool).await?;

    let allocator = UidAllocator::new(
        pool,
        service_config.preallocation_size,
        service_config.maximum_allocation_size,
        service_config.default_allocation_size,
    );

    let (_tx, rx) = tokio::sync::oneshot::channel();
    UidAllocatorServiceServer::builder(allocator, service_config.uid_allocator_bind_address, rx)
        .build()
        .serve()
        .await?;

    Ok(())
}
