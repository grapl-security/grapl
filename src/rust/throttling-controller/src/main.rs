use controller::ControllerError;
use rust_proto::graplinc::grapl::api::{
    throttling_controller::v1beta1::{
        server::{
            ThrottlingControllerApi,
            ThrottlingControllerServer,
        },
        ThrottlingRateForEventSourceRequest,
        ThrottlingRateForEventSourceResponse,
    },
    protocol::{
        error::ServeError,
        healthcheck::HealthcheckStatus,
        status::Status,
    },
};
use thiserror::Error;
use uuid::Uuid;

mod controller;
mod db;

#[derive(Debug, Error)]
#[non_exhaustive]
enum ControllerApiError {
    #[error("controller error {0}")]
    Controller(#[from] ControllerError),
}

impl From<ControllerApiError> for Status {
    fn from(error: ControllerApiError) -> Self {
        match error {
            ControllerApiError::Controller(e) => match e {
                ControllerError::Database(db_err) => Status::unavailable(
                    format!("database error {0}", db_err)
                ),
                ControllerError::Figment(fig_err) => Status::unavailable(
                    format!("configuration error {0}", fig_err)
                ),
                ControllerError::NotFound => Status::not_found("controller not found"),
            },
        }
    }
}

struct ControllerApi {
    controller: controller::Controller,
}

#[async_trait::async_trait]
impl ThrottlingControllerApi for ControllerApi {
    type Error = ControllerApiError;

    async fn throttling_rate_for_event_source(
        &self,
        request: ThrottlingRateForEventSourceRequest
    ) -> Result<ThrottlingRateForEventSourceResponse, ControllerApiError> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    println!("Hello, world!");

    Ok(())
}
