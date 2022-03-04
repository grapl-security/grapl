use std::convert::TryFrom;

use argon2::{
    password_hash::{
        rand_core::OsRng,
        SaltString,
    },
    PasswordHasher,
};
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::organization_management::{
    organization_management_service_server::{
        OrganizationManagementService,
        OrganizationManagementServiceServer,
    },
    CreateOrganizationRequest,
    CreateOrganizationRequestProto,
    CreateOrganizationResponse,
    CreateOrganizationResponseProto,
    CreateUserRequest,
    CreateUserRequestProto,
    CreateUserResponse,
    CreateUserResponseProto,
    OrganizationManagementDeserializationError,
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

use crate::OrganizationManagementServiceConfig;

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
            OrganizationManagementServiceError::Sql(e) => Status::internal(e.to_string()),
            OrganizationManagementServiceError::OrganizationManagementDeserializationError(e) => {
                Status::invalid_argument(e.to_string())
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct OrganizationManagement {
    pool: Pool<Postgres>,
}

impl OrganizationManagement {
    async fn try_from(
        service_config: &OrganizationManagementServiceConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
        let user_id = sqlx::types::Uuid::from_u128(Uuid::new_v4().as_u128());

        let CreateOrganizationRequest {
            organization_display_name,
            admin_username,
            admin_email,
            admin_password,
            should_reset_password,
        } = request;

        let mut transaction = self.pool.begin().await?;

        let password_hasher = argon2::Argon2::new(
            argon2::Algorithm::Argon2i,
            argon2::Version::V0x13,
            argon2::Params::new(102400, 2, 8, None)?,
        );

        let password = password_hasher
            .hash_password(&admin_password, &SaltString::generate(OsRng))?
            .serialize();

        sqlx::query!(
            r"
            INSERT INTO organizations (
                organization_id,
                display_name
            )
            VALUES ( $1, $2);
        ",
            organization_id,
            organization_display_name
        )
        .execute(&mut transaction)
        .await
        .map_err(OrganizationManagementServiceError::from)?;

        sqlx::query!(
            r"
            INSERT INTO users (
                user_id,
                organization_id,
                username,
                email,
                password,
                is_admin,
                should_reset_password
            )
             VALUES ( $1, $2, $3, $4, $5, $6, $7);
        ",
            user_id,
            organization_id,
            admin_username,
            admin_email,
            password.as_str(),
            true,
            should_reset_password
        )
        .execute(&mut transaction)
        .await
        .map_err(OrganizationManagementServiceError::from)?;

        transaction.commit().await?;

        Ok(CreateOrganizationResponse { organization_id })
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
            password,
        } = request;

        let password_hasher = argon2::Argon2::new(
            argon2::Algorithm::Argon2i,
            argon2::Version::V0x13,
            argon2::Params::new(102400, 2, 8, None)?,
        );

        let password = password_hasher
            .hash_password(&password, &SaltString::generate(OsRng))?
            .serialize();

        sqlx::query!(
            r"
             INSERT INTO users (
                user_id,
                organization_id,
                username,
                email,
                password,
                is_admin,
                should_reset_password
            )
             VALUES ( $1, $2, $3, $4, $5, $6, $7);
        ",
            user_id,
            organization_id,
            name,
            email,
            password.as_str(),
            false,
            true,
        )
        .execute(&self.pool)
        .await
        .map_err(OrganizationManagementServiceError::from)?;

        Ok(CreateUserResponse { user_id })
    }
}

#[tonic::async_trait]
impl OrganizationManagementService for OrganizationManagement {
    async fn create_organization(
        &self,
        request: Request<CreateOrganizationRequestProto>,
    ) -> Result<Response<CreateOrganizationResponseProto>, Status> {
        let request: CreateOrganizationRequestProto = request.into_inner();
        let request = CreateOrganizationRequest::try_from(request)
            .map_err(OrganizationManagementServiceError::from)?;

        let response = self.create_organization(request).await?;
        let response: CreateOrganizationResponseProto = response.into();
        Ok(Response::new(response))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequestProto>,
    ) -> Result<Response<CreateUserResponseProto>, Status> {
        let request: CreateUserRequestProto = request.into_inner();
        let request = CreateUserRequest::try_from(request)
            .map_err(OrganizationManagementServiceError::from)?;

        let response = self.create_user(request).await?;
        let response: CreateUserResponseProto = response.into();
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
        .add_service(OrganizationManagementServiceServer::new(
            organization_management,
        ))
        .serve(service_config.organization_management_bind_address)
        .await?;

    Ok(())
}