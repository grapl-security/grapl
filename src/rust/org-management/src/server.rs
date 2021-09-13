use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use crate::org_management::organization_manager_server::{OrganizationManager, OrganizationManagerServer};
use crate::org_management;
use org_management::{CreateOrgReply, CreateOrgRequest};
use sqlx::{Pool};
use sqlx::postgres::{PgPoolOptions, Postgres};


#[derive(Debug)]
pub struct OrganizationManagerRpc {
    pool: Pool<Postgres>,
}

#[tonic::async_trait]
impl OrganizationManager for OrganizationManagerRpc {
    async fn create_org(
        &self,
        request: Request<CreateOrgRequest>,
    ) -> Result<Response<CreateOrgReply>, Status> {
        println!("Org request data: {:?}", request); // don't actually print this

        let org_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();

        // store data in sql with new org id

        let reply = CreateOrgReply {
            organization_id: format!("Org Id {} Created", org_id).into(),
            admin_user_id: format!("Org Id {} Created", admin_id).into(),
        };

        Ok(Response::new(reply))
    }
}


#[tracing::instrument(err)]
pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:5502".parse()?;
    let pool =
        create_db_connection()
            .await?;
    let org = OrganizationManagerRpc { pool };

    tracing::info!(message="Listening on address", addr=?addr);
    Server::builder()
        .add_service(OrganizationManagerServer::new(org))
        .serve(addr)
        .await?;

    Ok(())
}

#[tracing::instrument(err)]
async fn create_db_connection() -> Result<Pool<Postgres>, sqlx::Error> {
    let url = std::env::var("POSTGRES_URL")
        .expect("POSTGRES_URL");

    tracing::info!(message="connecting to postgres", url=%url);
    // Create Connection Pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    // Insert Org Info
    Ok(pool)
}

