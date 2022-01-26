use crate::{
    orgmanagementlib::organization_manager_server::{
        OrganizationManagerServer,
    },
    create_db_conn::create_db_connection,
    organization_manager::{OrganizationManagerRpc},
};

use tonic::transport::Server;

#[tracing::instrument(err)]
pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("ORG_MANAGEMENT_PORT").expect("ORG_MANAGEMENT_PORT");
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let pool = create_db_connection().await?;

    let org = OrganizationManagerRpc { pool };

    tracing::info!(message="Listening on address", addr=?addr);

    Server::builder()
        .add_service(OrganizationManagerServer::new(org))
        .serve(addr)
        .await?;

    Ok(())
}

