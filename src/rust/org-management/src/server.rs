use std::convert::TryFrom;
use argon2::{Error, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use tonic::{
    transport::Server,
    Request,
    Response,
    Status,
};

use uuid::Uuid;

use rust_proto::org_management::organization_manager_service_server::{OrganizationManagerService, OrganizationManagerServiceServer};

use rust_proto::org_management::{CreateOrgRequest, CreateOrgRequestProto, CreateUserRequest, CreateUserRequestProto, ChangePasswordRequest, ChangePasswordRequestProto, CreateOrgResponse, CreateOrgResponseProto, CreateUserResponse, CreateUserResponseProto, ChangePasswordResponse, ChangePasswordResponseProto, OrgManagementDeserializationError};

use sqlx::{Pool};
use sqlx::postgres::{PgPoolOptions, Postgres};


#[derive(thiserror::Error, Debug)]
pub enum OrganizationManagerServiceError {
    #[error("Sql {0}")]
    Sql(#[from] sqlx::Error),
    #[error("OrgManagementDeserializationError {0}")]
    OrgManagementDeserializationError(#[from] OrgManagementDeserializationError),
    #[error("HashError {0}")]
    HashError(String),
}

impl From<argon2::Error> for OrganizationManagerServiceError {
    fn from(err: argon2::Error) -> Self {
        Self::HashError(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for OrganizationManagerServiceError {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::HashError(err.to_string())
    }
}



impl From<OrganizationManagerServiceError> for Status {
    fn from(e: OrganizationManagerServiceError) -> Self {
        match e {
            OrganizationManagerServiceError::Sql(e) => {
                Status::internal(e.to_string())
            }
            OrganizationManagerServiceError::OrgManagementDeserializationError(e) => {
                Status::invalid_argument(e.to_string())
            }
            _ => todo!()
        }
    }
}

struct Password {
    password: String,
}

#[derive(Debug)]
pub struct OrganizationManager {
    pool: Pool<Postgres>,
}

impl OrganizationManager {
    async fn create_org(
        &self,
        request: CreateOrgRequest,
    ) -> Result<CreateOrgResponse, OrganizationManagerServiceError> {
        let org_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());

        let CreateOrgRequest {
            org_display_name,
            admin_username,
            admin_email,
            admin_password,
            should_reset_password,
        } = request;

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query!(r"
            INSERT INTO organization (
                org_id,
                display_name
            )
             VALUES ( $1, $2 );
        ",
                org_id,
                org_display_name,
        )
            .execute(&mut transaction)
            .await
            .map_err(OrganizationManagerServiceError::from)?;

        let password_hasher = argon2::Argon2::new(
            argon2::Algorithm::Argon2i,
            argon2::Version::V0x13,
            argon2::Params::new(102400, 2, 8, None)?,
        );

        let admin_password = password_hasher.hash_password(
            &admin_password,
            &SaltString::generate(OsRng),
        )?.serialize();

        sqlx::query!(r"
            INSERT INTO users (
                org_id,
                username,
                email,
                password,
                should_reset_password
            )
            VALUES ( $1, $2, $3, $4, $5 );
        ",
                org_id,
                admin_username,
                admin_email,
                admin_password.as_str(),
                should_reset_password,
        )
            .execute(&mut transaction)
            .await
            .map_err(OrganizationManagerServiceError::from)?;

        transaction.commit().await?;

        Ok(CreateOrgResponse {})
    }

    async fn create_user(
        &self,
        request: CreateUserRequest,
    ) -> Result<CreateUserResponse, OrganizationManagerServiceError> {
        let user_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());

        let CreateUserRequest {
            organization_id, // we need to do a lookup here
            name,
            email,
            password
        } = request;

        let password_hasher = argon2::Argon2::new(
            argon2::Algorithm::Argon2i,
            argon2::Version::V0x13,
            argon2::Params::new(102400, 2, 8, None)?,
        );

        let password = password_hasher.hash_password(
            &password,
            &SaltString::generate(OsRng),
        )?.serialize();


        let row = sqlx::query!(r"
            INSERT INTO users (
                user_id,
                org_id,
                username,
                email,
                password
            )
             VALUES ( $1, $2, $3, $4, $5 )
        ",
                user_id,
                organization_id,
                name,
                email,
                password.as_str()
        )
            .execute(&self.pool)
            .await
            .map_err(OrganizationManagerServiceError::from)?;
        //
        // if row.rows_affected() == 0 {
        //     return Err(Status::internal("User was not created successfully"));
        // }
        //
        Ok(CreateUserResponse {})
    }

    async fn change_password(
        &self,
        request: ChangePasswordRequest,
    ) -> Result<ChangePasswordResponse, OrganizationManagerServiceError> {
        let ChangePasswordRequest {
            organization_id,
            user_id,
            old_password,
            new_password,
        } = request;

        let user_id = sqlx::types::Uuid::from_u128(user_id.as_u128());

        let stored_password_hash = sqlx::query_as!(
            Password,
            r"SELECT password
            FROM users
            WHERE user_id = $1;",
                 &user_id,
        )
            .fetch_one(&self.pool)
            .await
            .map_err(OrganizationManagerServiceError::from)?
            .password;

        let password_hasher = argon2::Argon2::new(
            argon2::Algorithm::Argon2i,
            argon2::Version::V0x13,
            argon2::Params::new(102400, 2, 8, None)?,
        );

        let hash = argon2::PasswordHash::new(&stored_password_hash)?;

        // return early if mismatch
        password_hasher.verify_password(&old_password, &hash)?;

        let password = password_hasher.hash_password(
            &new_password,
            &SaltString::generate(OsRng),
        )?.serialize();

        let row = sqlx::query!(
            "UPDATE users SET password = $2 WHERE user_id = $1",
                &user_id,
                &password.as_str()
        )
            .execute(&self.pool)
            .await
            .map_err(OrganizationManagerServiceError::from)?;

        Ok(ChangePasswordResponse {})
    }
}

#[tonic::async_trait]
impl OrganizationManagerService for OrganizationManager {
    async fn create_org(
        &self,
        request: Request<CreateOrgRequestProto>,
    ) -> Result<Response<CreateOrgResponseProto>, Status> {
        let request: CreateOrgRequestProto = request.into_inner();
        let request =
            CreateOrgRequest::try_from(request).map_err(OrganizationManagerServiceError::from)?;

        let response = self.create_org(request).await?;
        let response: CreateOrgResponseProto = response.into();
        Ok(Response::new(response))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequestProto>,
    ) -> Result<Response<CreateUserResponseProto>, Status> {
        let request: CreateUserRequestProto = request.into_inner();
        let request =
            CreateUserRequest::try_from(request).map_err(OrganizationManagerServiceError::from)?;

        let response = self.create_user(request).await?;
        let response: CreateUserResponseProto = response.into();
        Ok(Response::new(response))
    }

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequestProto>,
    ) -> Result<Response<ChangePasswordResponseProto>, Status> {
        let request: ChangePasswordRequestProto = request.into_inner();
        let request =
            ChangePasswordRequest::try_from(request).map_err(OrganizationManagerServiceError::from)?;

        let response = self.change_password(request).await?;
        let response: ChangePasswordResponseProto = response.into();
        Ok(Response::new(response))
    }
}
//
//
// #[tracing::instrument(err)]
// pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "0.0.0.0:5502".parse()?;
//     let pool =
//         create_db_connection()
//             .await?;
//
//     let org = OrganizationManager { pool };
//
//     tracing::info!(message="Listening on address", addr=?addr);
//
//     Server::builder()
//         .add_service(OrganizationManagerServiceServer::new(org))
//         .serve(addr)
//         .await?;
//
//     Ok(())
// }

#[tracing::instrument(err)]
async fn create_db_connection() -> Result<Pool<Postgres>, sqlx::Error> {
    // let url = std::env::var("DATABASE_URL")
    //     .expect("DATABASE_URL");
    let url = "postgres://postgres@localhost?db_name=postgres&user=postgres&password=postgres";

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

