use std::sync::Arc;

use clap::Parser;
use rust_proto::graplinc::grapl::api::lens_manager::v1beta1::server::LensManagerServiceServer;
use scylla::CachingSession;

use crate::{
    config::LensManagerServiceConfig,
    server::LensManager,
};

mod config;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LensManagerServiceConfig::parse();
    let mut scylla_config = scylla::SessionConfig::new();
    scylla_config.add_known_nodes_addr(&config.graph_db_config.graph_db_addresses[..]);
    scylla_config.auth_username = Some(config.graph_db_config.graph_db_auth_username.to_owned());
    scylla_config.auth_password = Some(config.graph_db_config.graph_db_auth_password.to_owned());

    let scylla_client = Arc::new(CachingSession::from(
        scylla::Session::connect(scylla_config).await?,
        10_000,
    ));
    let lens_manager_service = LensManager::new(scylla_client);

    let (_tx, rx) = tokio::sync::oneshot::channel();
    LensManagerServiceServer::builder(
        lens_manager_service,
        config.lens_manager_service_bind_address,
        rx,
    )
    .build()
    .serve()
    .await?;

    Ok(())
}
