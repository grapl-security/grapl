use std::convert::TryFrom;

use argon2::{PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use sqlx::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};
use tonic::{
    Request,
    Response,
    Status,
    transport::Server,
};
use uuid::Uuid;

use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::organization_management::{
    ChangePasswordRequest,
    ChangePasswordRequestProto,
    ChangePasswordResponse,
    ChangePasswordResponseProto,
    CreateOrganizationRequest,
    CreateOrganizationRequestProto,
    CreateOrganizationResponse,
    CreateOrganizationResponseProto,
    CreateUserRequest,
    CreateUserRequestProto,
    CreateUserResponse,
    CreateUserResponseProto,
    organization_management_service_server::{OrganizationManagementService, OrganizationManagementServiceServer},
    OrganizationManagementDeserializationError,
};

use crate::{
    OrganizationManagementServiceConfig,
};

#[derive(thiserror::Error, Debug)]
pub enum OrganizationManagementServiceError {
    #[error("Sql {0}")]
    Sql(#[from] sqlx::Error),
    #[error("OrganizationanizationManagementDeserializationError {0}")]
    OrganizationManagementDeserializationError(#[from] OrganizationManagementDeserializationError),
    #[error("HashError {0}")]
    HashError(String),
}

impl From<argon2::Error> for OrganizationManagementServiceError {
    fn from(err: argon2::Error) -> Self {
        Self::HashError(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for OrganizationManagementServiceError {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::HashError(err.to_string())
    }
}


impl From<OrganizationManagementServiceError> for Status {
    fn from(e: OrganizationManagementServiceError) -> Self {
        match e {
            OrganizationManagementServiceError::Sql(e) => {
                Status::internal(e.to_string())
            }
            OrganizationManagementServiceError::OrganizationManagementDeserializationError(e) => {
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
pub struct OrganizationManagement {
    pool: Pool<Postgres>,
}

impl OrganizationManagement {
    async fn try_from(service_config: &OrganizationManagementServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let postgres_address = format!(
            "postgresql://{}:{}@{}:{}",
            service_config.organization_management_db_username,
            service_config.organization_management_db_password,
            service_config.organization_management_db_hostname,
            service_config.organization_management_db_port,
        );

        Ok(Self {
            pool: sqlx::PgPool::connect(&postgres_address)
                .timeout(std::time::Duration::from_secs(5))
                .await??,
        })
    }
    async fn create_organization(
        &self,
        request: CreateOrganizationRequest,
    ) -> Result<CreateOrganizationResponse, OrganizationManagementServiceError> {
        let organization_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());

        let CreateOrganizationRequest {
            organization_display_name,
            admin_username,
            admin_email,
            admin_password,
            should_reset_password,
        } = request;

        let mut transaction = self.pool.begin().await?;

        sqlx::query!(r"
            INSERT INTO organization (
                organization_id,
                display_name
            )
             VALUES ( $1, $2 );
        ",
                organization_id,
                organization_display_name,
        )
            .execute(&mut transaction)
            .await
            .map_err(OrganizationManagementServiceError::from)?;

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
                organization_id,
                username,
                email,
                password,
                should_reset_password
            )
            VALUES ( $1, $2, $3, $4, $5 );
        ",
                organization_id,
                admin_username,
                admin_email,
                admin_password.as_str(),
                should_reset_password,
        )
            .execute(&mut transaction)
            .await
            .map_err(OrganizationManagementServiceError::from)?;

        transaction.commit().await?;

        Ok(CreateOrganizationResponse {})
    }

    async fn create_user(
        &self,
        request: CreateUserRequest,
    ) -> Result<CreateUserResponse, OrganizationManagementServiceError> {
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


        sqlx::query!(r"
            INSERT INTO users (
                user_id,
                organization_id,
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
            .map_err(OrganizationManagementServiceError::from)?;

        Ok(CreateUserResponse {})
    }

    async fn change_password(
        &self,
        request: ChangePasswordRequest,
    ) -> Result<ChangePasswordResponse, OrganizationManagementServiceError> {
        let ChangePasswordRequest {
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
            .map_err(OrganizationManagementServiceError::from)?
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

        sqlx::query!(
            "UPDATE users SET password = $2 WHERE user_id = $1",
                &user_id,
                &password.as_str()
        )
            .execute(&self.pool)
            .await
            .map_err(OrganizationManagementServiceError::from)?;

        Ok(ChangePasswordResponse {})
    }
}

#[tonic::async_trait]
impl OrganizationManagementService for OrganizationManagement {
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequestProto>,
    ) -> Result<Response<CreateOrganizationResponseProto>, Status> {
        let request: CreateOrganizationRequestProto = request.into_inner();
        let request =
            CreateOrganizationRequest::try_from(request).map_err(OrganizationManagementServiceError::from)?;

        let response = self.create_organization(request).await?;
        let response: CreateOrganizationResponseProto = response.into();
        Ok(Response::new(response))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequestProto>,
    ) -> Result<Response<CreateUserResponseProto>, Status> {
        let request: CreateUserRequestProto = request.into_inner();
        let request =
            CreateUserRequest::try_from(request).map_err(OrganizationManagementServiceError::from)?;

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
            ChangePasswordRequest::try_from(request).map_err(OrganizationManagementServiceError::from)?;

        let response = self.change_password(request).await?;
        let response: ChangePasswordResponseProto = response.into();
        Ok(Response::new(response))
    }
}

#[tracing::instrument(err)]
async fn create_db_connection() -> Result<Pool<Postgres>, sqlx::Error> {
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

pub async fn exec_service(
    service_config: OrganizationManagementServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization_management = OrganizationManagement::try_from(&service_config).await?;

    tracing::info!(message = "Performing migration",);

    sqlx::migrate!().run(&organization_management.pool).await?;

    tracing::info!(message = "Binding service",);

    Server::builder()
        .trace_fn(|request| {
            tracing::info_span!(
                "OrganizationManagement",
                headers = ?request.headers(),
                method = ?request.method(),
                uri = %request.uri(),
                extensions = ?request.extensions(),
            )
        })
        .add_service(OrganizationManagementServiceServer::new(organization_management))
        .serve(service_config.organization_management_bind_address)
        .await?;

    Ok(())
}
