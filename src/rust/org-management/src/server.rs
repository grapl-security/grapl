use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use crate::org_management::organization_manager_server::{OrganizationManager, OrganizationManagerServer};

use crate::org_management;

use org_management::{
    CreateOrgReply,
    CreateOrgRequest,
    CreateUserRequest,
    CreateUserReply,
    ChangePasswordRequest,
    ChangePasswordReply
};

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
        println!("Org request data: {:?}", &request); // don't actually print this


        let org_id = Uuid::new_v4();
        let admin_id = Uuid::new_v4();

        let CreateOrgRequest {
            org_display_name,
            admin_username,
            admin_email,
            admin_password,
            should_reset_password,
        } = &request.into_inner();

        // let conn = create_db_connection();


        // store data in sql with new org id

        // let reply = CreateOrgReply {
        //     organization_id: format!("Org Id {} Created", org_id).into(),
        //     admin_user_id: format!("Org Id {} Created", admin_id).into(),
        // };

        // let row: (i64, ) = sqlx::query_as(
        // "insert into organization (name) values ($1) returning id"
        // )
        //     .bind("new organization")
        //     .fetch_one(self.pool)
        //     .await?;
        //

        let number_of_rows_effected = sqlx::query_as(
            "INSERT INTO organization ( org_display_name, admin_username, admin_email, admin_password, should_reset_password) VALUES ($1, $2, $3, $4, $5)",
        )
            .bind("new organization")
            .fetch_one(&self.pool)
            .await?;


        // let number_of_rows_affected = &conn.execute(
        //     "INSERT INTO organization ( org_display_name, admin_username, admin_email, admin_password, should_reset_password) VALUES ($1, $2, $3, $4, $5)",
        //     &[
        //         &org_display_name,
        //         &admin_username,
        //         &admin_email,
        //         &admin_password,
        //         &should_reset_password,
        //     ]
        // )
        //     .unwrap();

        let reply = if number_of_rows_effected == &(0 as u64) {
            CreateOrgReply {
                message: format!(
                    "Fail to create org with id {}.",
                    &org_id
                ),
            }
        } else {
            CreateOrgReply {
                message: format!(
                    "Create {} org with id {}.",
                    &number_of_rows_effected, &org_id
                ),
            }
        };

        Ok(Response::new(reply))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status>{
        println!("Org request data: {:?}", &request); // don't actually print this

        let org_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let user_reply = CreateUserReply{
            organization_id: format!("Org ID {} Created", org_id).into(),
            user_id: format!("Org Id {} Created", user_id).into()
        };

        Ok(Response::new(user_reply))
    }

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordReply>, Status>{
        println!("Changed password for user x: {:?}", request); // don't actually print this

        // alter sql
        let temp_pass = "true";

        let password_reply = ChangePasswordReply{
            changed_password: format!("Org Id {} Created", temp_pass)
        };

        Ok(Response::new(password_reply))
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

