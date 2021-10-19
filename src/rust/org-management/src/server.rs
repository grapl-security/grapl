use org_management::{
    ChangePasswordRequest,
    CreateOrgRequest,
    CreateUserRequest,
    EmptyResp,
};
use sqlx::{
    postgres::{
        PgPoolOptions,
        Postgres,
    },
    Pool,
};
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};
use uuid::Uuid;

use crate::{
    org_management,
    org_management::organization_manager_server::{
        OrganizationManager,
        OrganizationManagerServer,
    },
};

#[derive(Debug)]
pub struct OrganizationManagerRpc {
    pool: Pool<Postgres>,
}

#[derive(thiserror::Error, Debug)]
pub enum OrganizationManagerError {
    #[error("sql")]
    Sql(#[from] sqlx::Error),
}

impl From<OrganizationManagerError> for Status {
    fn from(e: OrganizationManagerError) -> Self {
        match e {
            OrganizationManagerError::Sql(e) => Status::internal(e.to_string()),
        }
    }
}

#[tonic::async_trait]
impl OrganizationManager for OrganizationManagerRpc {
    async fn create_org(
        &self,
        request: Request<CreateOrgRequest>,
    ) -> Result<Response<EmptyResp>, Status> {
        println!("Org request data: {:?}", &request);

        let org_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());
        // let user_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());

        let CreateOrgRequest {
            org_display_name,
            admin_username,
            admin_email,
            admin_password,
            should_reset_password,
        } = &request.into_inner();

        let row = sqlx::query(
            r"
            INSERT INTO organization (
                org_id,
                org_display_name,
                admin_username,
                admin_email,
                admin_password,
                should_reset_password
            )
             VALUES ( $1, $2, $3, $4, $5, $6 )
        ",
        )
        .bind(org_id)
        .bind(org_display_name)
        .bind(admin_username)
        .bind(admin_email)
        .bind(admin_password)
        .bind(should_reset_password)
        .execute(&self.pool)
        .await
        .map_err(OrganizationManagerError::from)?;

        if row.rows_affected() == 0 {
            return Err(Status::internal(
                "Organization was not created successfully",
            ));
        }

        Ok(Response::new(EmptyResp {}))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<EmptyResp>, Status> {
        println!("Org request data: {:?}", &request); // don't actually print this

        let user_id = Uuid::new_v4().to_string();

        let CreateUserRequest {
            organization_id, // we need to do a lookup here
            name,
            email,
            password,
        } = &request.into_inner();

        let row = sqlx::query(
            r"
            INSERT INTO users (
                user_id,
                org_id,
                name,
                email,
                password
            )
             VALUES ( $1, $2, $3, $4, $5 )
        ",
        )
        .bind(user_id)
        .bind(organization_id)
        .bind(name)
        .bind(email)
        .bind(password)
        .execute(&self.pool)
        .await
        .map_err(OrganizationManagerError::from)?;

        if row.rows_affected() == 0 {
            return Err(Status::internal("User was not created successfully"));
        }

        Ok(Response::new(EmptyResp {}))
    }

    async fn change_password(
        &self,
        _request: Request<ChangePasswordRequest>,
    ) -> Result<Response<EmptyResp>, Status> {
        // println!("Changed password for user x: {:?}", request); // don't actually print this

        // check to see if old password matches what we have in db
        // if it passes, update with new password
        // let row = sqlx::query!(
        //     "UPDATE users SET password = $2 WHERE user_id = $1",
        //          &user_id,
        //         &organization_id,
        //         &old_password,
        //         &new_password
        // )
        //     .bind("new user")
        //     .execute(&self.pool)
        //     .await
        //     .map_err(OrganizationManagerError::from)?;
        //
        // if row.rows_affected() == 0 {
        //     return Err(Status::internal("Organization was not created successfully"));
        // }

        Ok(Response::new(EmptyResp {}))
    }
}

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

#[tracing::instrument(err)]
async fn create_db_connection() -> Result<Pool<Postgres>, sqlx::Error> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");

    println!("databse url {}", url);

    tracing::info!(message="connecting to postgres", url=%url);
    // Create Connection Pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    // Insert Org Info
    Ok(pool)
}
