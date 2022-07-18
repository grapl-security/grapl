mod config;
mod notify_broadcast;
mod service;

use clap::Parser;
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::graplinc::grapl::api::lens_subscription_service::v1beta1::server::LensSubscriptionServiceServer;

use crate::{
    config::LensSubscriptionServiceConfig,
    service::LensSubscriptionService,
};

#[tracing::instrument]
async fn inner_main() -> Result<(), Box<dyn std::error::Error>> {
    let service_config = LensSubscriptionServiceConfig::parse();

    let lens_db_config = service_config.db_config;
    let postgres_address = format!(
        "postgresql://{}:{}@{}:{}",
        lens_db_config.lens_subscription_service_db_username,
        lens_db_config.lens_subscription_service_db_password,
        lens_db_config.lens_subscription_service_db_hostname,
        lens_db_config.lens_subscription_service_db_port,
    );

    let pool = sqlx::PgPool::connect(&postgres_address)
        .timeout(std::time::Duration::from_secs(5))
        .await??;

    let lens_subscription_service = LensSubscriptionService::new(pool).await?;

    let (_tx, rx) = tokio::sync::oneshot::channel();
    LensSubscriptionServiceServer::builder(
        lens_subscription_service,
        service_config.lens_subscription_service_bind_address,
        rx,
    )
    .build()
    .serve()
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_env, _guard) = grapl_config::init_grapl_env!();

    inner_main().await
}
