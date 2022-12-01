use std::time::SystemTime;

use figment::{
    Provider,
    Figment,
};
use grapl_config::PostgresClient;
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub(crate) struct Controller {
    event_source_id: Uuid,
    plugin_id: Uuid,
    is_generator: bool,
    K: f64,
    T_i: f64,
    T_d: f64,
    T_t: f64,
    N: f64,
    b: f64,
    P_k1: f64,
    I_k1: f64,
    D_k1: f64,
    r_k1: f64,
    y_k1: f64,
    y_k2: f64,
    t_k1: SystemTime,
}

#[allow(non_snake_case)]
impl Controller {
    pub fn new(
        event_source_id: Uuid,
        plugin_id: Uuid,
        is_generator: bool,
        K: f64,
        T_i: f64,
        T_d: f64,
        T_t: f64,
        N: f64,
        b: f64,
        P_k1: f64,
        I_k1: f64,
        D_k1: f64,
        r_k1: f64,
        y_k1: f64,
        y_k2: f64,
        t_k1: SystemTime,
    ) -> Self {
        Self {
            event_source_id,
            plugin_id,
            is_generator,
            K,
            T_i,
            T_d,
            T_t,
            N,
            b,
            P_k1,
            I_k1,
            D_k1,
            r_k1,
            y_k1,
            y_k2,
            t_k1,
        }
    }

    pub fn event_source_id(&self) -> Uuid {
        self.event_source_id
    }

    pub fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }

    pub fn is_generator(&self) -> bool {
        self.is_generator
    }

    pub fn K(&self) -> f64 {
        self.K
    }

    pub fn T_i(&self) -> f64 {
        self.T_i
    }

    pub fn T_d(&self) -> f64 {
        self.T_d
    }

    pub fn T_t(&self) -> f64 {
        self.T_t
    }

    pub fn N(&self) -> f64 {
        self.N
    }

    pub fn b(&self) -> f64 {
        self.b
    }

    pub fn P_k1(&self) -> f64 {
        self.P_k1
    }

    pub fn I_k1(&self) -> f64 {
        self.I_k1
    }

    pub fn D_k1(&self) -> f64 {
        self.D_k1
    }

    pub fn r_k1(&self) -> f64 {
        self.r_k1
    }

    pub fn y_k1(&self) -> f64 {
        self.y_k1
    }

    pub fn y_k2(&self) -> f64 {
        self.y_k2
    }

    pub fn t_k1(&self) -> SystemTime {
        self.t_k1
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigurationError {
    #[error("error extracting figment config {0}")]
    Figment(#[from] figment::Error),
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ControllerDbConfiguration {
    address: String,
    username: String,
    password: grapl_config::SecretString,
}

impl ControllerDbConfiguration {
    pub fn new(
        address: String,
        username: String,
        password: grapl_config::SecretString,
    ) -> Self {
        Self { address, username, password }
    }

    pub fn from<T>(provider: T) -> Result<Self, ConfigurationError>
    where
        T: Provider
    {
        Ok(Figment::from(provider).extract()?)
    }

    pub(crate) fn address(&self) -> &str {
        &self.address
    }

    pub(crate) fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn password(&self) -> grapl_config::SecretString {
        self.password.clone()
    }
}

impl grapl_config::ToPostgresUrl for ControllerDbConfiguration {
    fn to_postgres_url(self) -> grapl_config::PostgresUrl {
        grapl_config::PostgresUrl {
            address: self.address().to_owned(),
            username: self.username().to_owned(),
            password: self.password(),
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub(crate) enum ControllerDbError {
    #[error("sqlx migration error {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("initialization error {0}")]
    Initialization(#[from] grapl_config::PostgresDbInitError),
}

#[derive(Clone)]
pub(crate) struct ControllerDb {
    pool: sqlx::PgPool
}

#[async_trait::async_trait]
impl PostgresClient for ControllerDb {
    type Config = ControllerDbConfiguration;
    type Error = ControllerDbError;

    fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }

    #[tracing::instrument]
    async fn migrate(pool: &sqlx::Pool<sqlx::Postgres>) -> Result<(), sqlx::migrate::MigrateError> {
        tracing::info!(message = "Performing database migration");

        sqlx::migrate!().run(pool).await
    }
}

impl ControllerDb {
    #[tracing::instrument(skip(self), err)]
    pub async fn create_or_update_controller(
        &self,
        controller: Controller,
    ) -> Result<(), ControllerDbError> {
        todo!()
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn get_controller(
        &self,
        event_source_id: Uuid,
        plugin_id: Uuid,
    ) -> Result<Option<Controller>, ControllerDbError> {
        todo!()
    }
}
