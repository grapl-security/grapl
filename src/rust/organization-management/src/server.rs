use std::time::Duration;

use argon2::{
    password_hash::{
        rand_core::OsRng,
        SaltString,
    },
    PasswordHasher,
};
use grapl_utils::future_ext::GraplFutureExt;
use rust_proto::{
    graplinc::grapl::api::organization_management::v1beta1::{
        server::{
            ConfigurationError as ServerConfigurationError,
            OrganizationManagementApi,
            OrganizationManagementServer,
        },
        CreateOrganizationRequest,
        CreateOrganizationResponse,
        CreateUserRequest,
        CreateUserResponse,
    },
    protocol::{
        healthcheck::HealthcheckStatus,
        status::Status,
    },
};
use sqlx::{
    postgres::Postgres,
    Pool,
};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::OrganizationManagementServiceConfig;

#[derive(thiserror::Error, Debug)]
pub enum OrganizationManagementServiceError {
    #[error("Sql {0}")]
    Sql(#[from] sqlx::Error),
    #[error("HashError {0}")]
    HashError(String),
    #[error("ServerError {0}")]
    ServerError(#[from] ServerConfigurationError),
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
                .timeout(Duration::from_secs(5))
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

pub struct ManagementApi {
    organization_management: OrganizationManagement,
}

impl ManagementApi {
    pub fn new(organization_management: OrganizationManagement) -> Self {
        ManagementApi {
            organization_management,
        }
    }
}

#[async_trait::async_trait]
impl OrganizationManagementApi for ManagementApi {
    type Error = OrganizationManagementServiceError;

    async fn create_organization(
        &self,
        request: CreateOrganizationRequest,
    ) -> Result<CreateOrganizationResponse, Self::Error> {
        self.organization_management
            .create_organization(request)
            .await
    }

    async fn create_user(
        &self,
        request: CreateUserRequest,
    ) -> Result<CreateUserResponse, Self::Error> {
        self.organization_management.create_user(request).await
    }
}

pub async fn exec_service(
    service_config: OrganizationManagementServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let organization_management = OrganizationManagement::try_from(&service_config).await?;

    tracing::info!(message = "Performing migration",);

    sqlx::migrate!().run(&organization_management.pool).await?;

    tracing::info!(message = "Binding service",);

    let (server, _shutdown_tx) = OrganizationManagementServer::new(
        ManagementApi::new(organization_management),
        TcpListener::bind(service_config.organization_management_bind_address).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(
            service_config.organization_management_healthcheck_polling_interval_ms,
        ),
    );

    Ok(server.serve().await?)
}
