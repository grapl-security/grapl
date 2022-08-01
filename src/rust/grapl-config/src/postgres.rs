use grapl_utils::future_ext::GraplFutureExt;
use secrecy::ExposeSecret;

#[derive(Debug, Clone)]
pub struct PostgresUrl {
    pub address: String,
    pub username: String,
    pub password: super::SecretString,
}

impl std::fmt::Display for PostgresUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "postgresql://{username}:<REDACTED>@{address}",
            username = self.username,
            address = self.address
        )
    }
}

impl PostgresUrl {
    /// Connect to Postgres with a timeout of 5 seconds.
    pub async fn connect(&self) -> Result<sqlx::Pool<sqlx::Postgres>, PostgresDbInitError> {
        let timeout = std::time::Duration::from_secs(5);

        self.connect_with_timeout(timeout).await
    }

    /// Connect to Postgres with the supplied timeout.
    #[tracing::instrument]
    pub async fn connect_with_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<sqlx::Pool<sqlx::Postgres>, PostgresDbInitError> {
        tracing::info!(message = "Connecting to database", postgres_url =% self, ?timeout);

        let pool = sqlx::PgPool::connect(
            format!(
                "postgresql://{username}:{password}@{address}",
                username = self.username,
                password = self.password.expose_secret(),
                address = self.address
            )
            .as_str(),
        )
        .timeout(timeout)
        .await
        .map_err(|_| PostgresDbInitError::Timeout { timeout })??;

        Ok(pool)
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum PostgresDbInitError {
    #[error("sqlx migration error: {0}")]
    MigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("unable to connect: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("connection timeout after {timeout:?}")]
    Timeout { timeout: std::time::Duration },
}

/// A trait for deriving a PostgresUrl, which is used by the PostgresClient trait. This is
/// intended to be used by Postgres configs.
///
/// Ideally we could have a single Postgres config type that all Postgres clients could use,
/// instead of needing this trait, but we can't exactly do that right now while keeping consistent
/// with Grapl's use of clap derive for argument processing.
///
/// The clap::Parser derive macro does not support supllied prefixes when flattening a config
/// structure. This means all fields of the Postgres config would need to be same across all uses
/// of it (ex: DB_ADDRESS). At the time of this writing that wouldn't be a problem for running our
/// services themselves, because 1) we don't have any services that uses multiple Postgres servers,
/// and 2) we could set the common name in our grapl-core.nomad for each service (ex: DB_USERNAME
/// = var.example.username). However, our Rust integration tests are all running in a single
/// environment, and those tests need to use the same config builders as the services themselves.
/// So for now we need to have the argument variable names include uniqueness, such as prefixing
/// with the service's name (EXAMPLE_DB_HOSTNAME), meaning we can't use a common config. It looks
/// like this issue is being tracked at https://github.com/clap-rs/clap/issues/3513.
///
/// We could stray from using clap for this and have code that takes a prefix for loading Postgres
/// environment configs, but this isn't great for a couple reasons:
/// 1. It hides required arguments for a service in a different package. Some services have a
///    src/config.rs, which clearly defines all the parameters for the service. But by not using
///    clap here we'd break that for users, and we'd break the help formatter produced by clap when
///    one of the clap arguments are not supplied.
/// 2. Clap has good error reporting when required arguments are not supplied. We'd have to
///    recreate that, which isn't a big deal, but still.
pub trait ToPostgresUrl {
    fn to_postgres_url(self) -> PostgresUrl;
}

#[async_trait::async_trait]
pub trait PostgresClient {
    type Config: ToPostgresUrl + Send + std::fmt::Debug;
    type Error: From<PostgresDbInitError>;

    fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self;

    /// Initialize the Postgres client by connecting to Postrges and running a migration on the server.
    #[tracing::instrument]
    async fn init_with_config(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let postgres_url = config.to_postgres_url();
        let pool = postgres_url.connect().await?;

        Self::migrate(&pool).await.map_err(|e| e.into())?;

        Ok(Self::new(pool))
    }

    /// Run a migration to initialize a Postgres
    async fn migrate(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<(), sqlx::migrate::MigrateError>;
}
