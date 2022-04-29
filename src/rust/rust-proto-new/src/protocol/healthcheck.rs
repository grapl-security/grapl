#![allow(warnings)]

use futures::{
    FutureExt,
    TryFutureExt,
};
use thiserror::Error;
use tokio::time::error::Elapsed;
use tonic::Request;
use tonic_health::proto::{
    health_check_response::ServingStatus as ServingStatusProto,
    health_client::HealthClient as HealthClientProto,
    HealthCheckRequest as HealthCheckRequestProto,
};

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum HealthcheckError {
    #[error("not found {0}")]
    NotFound(String),

    #[error("healthcheck failed {0}")]
    HealthcheckFailed(String),
}

#[non_exhaustive]
#[derive(Debug)]
pub enum HealthcheckStatus {
    Serving,
    NotServing,
    Unknown,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("failed to connect {0}")]
    ConnectionError(#[from] tonic::transport::Error),

    #[error("healthcheck failed {0}")]
    HealtcheckFailed(#[from] HealthcheckError),

    #[error("timeout elapsed {0}")]
    TimeoutElapsed(#[from] Elapsed),
}

pub mod client {
    use std::time::Duration;

    use super::*;

    pub struct HealthcheckClient {
        proto_client: HealthClientProto<tonic::transport::Channel>,
        service_name: &'static str,
    }

    impl HealthcheckClient {
        #[tracing::instrument]
        pub async fn connect<T>(
            endpoint: T,
            service_name: &'static str,
        ) -> Result<Self, ConfigurationError>
        where
            T: std::convert::TryInto<tonic::transport::Endpoint> + std::fmt::Debug,
            T::Error: std::error::Error + Send + Sync + 'static,
        {
            Ok(HealthcheckClient {
                proto_client: HealthClientProto::connect(endpoint).await?,
                service_name,
            })
        }

        #[tracing::instrument(skip(self))]
        pub async fn check_health(&mut self) -> Result<HealthcheckStatus, HealthcheckError> {
            let request = HealthCheckRequestProto {
                service: self.service_name.to_string(),
            };

            let response = match self.proto_client.check(request).await {
                Ok(response) => response.into_inner(),
                Err(e) => match e.code() {
                    tonic::Code::NotFound => return Err(HealthcheckError::NotFound(e.to_string())),
                    _ => return Err(HealthcheckError::HealthcheckFailed(e.to_string())),
                },
            };

            match response.status() {
                ServingStatusProto::Serving => Ok(HealthcheckStatus::Serving),
                ServingStatusProto::NotServing => Ok(HealthcheckStatus::NotServing),
                ServingStatusProto::Unknown => Ok(HealthcheckStatus::Unknown),
                ServingStatusProto::ServiceUnknown => Err(HealthcheckError::HealthcheckFailed(
                    "service unknown".to_string(),
                )),
            }
        }

        #[tracing::instrument]
        pub async fn wait_until_healthy<T>(
            endpoint: T,
            service_name: &'static str,
            timeout: Duration,
            polling_interval: Duration,
        ) -> Result<Self, ConfigurationError>
        where
            T: std::convert::TryInto<tonic::transport::Endpoint> + Clone + std::fmt::Debug,
            T::Error: std::error::Error + Send + Sync + 'static,
        {
            let client_fut = async move {
                let mut healthcheck_client = loop {
                    match HealthcheckClient::connect(endpoint.clone(), service_name).await {
                        Ok(client) => break client,
                        Err(e) => {
                            tracing::warn!(
                                message="could not construct healthcheck client",
                                service_name=%service_name,
                                polling_interval=%polling_interval.as_millis(),
                                error=?e,
                            );
                            tokio::time::sleep(polling_interval).await;
                        }
                    }
                };

                loop {
                    match healthcheck_client.check_health().await {
                        Ok(result) => match result {
                            HealthcheckStatus::Serving => {
                                tracing::info!(message="serving requests", service_name=%service_name);
                                break Ok(healthcheck_client);
                            }
                            other => {
                                tracing::warn!(
                                    message="not yet serving requests",
                                    service_name=%service_name,
                                    polling_interval=%polling_interval.as_millis(),
                                    other=?other,
                                );
                                tokio::time::sleep(polling_interval).await;
                            }
                        },
                        Err(e) => match e {
                            HealthcheckError::HealthcheckFailed(_) => break Err(e),
                            HealthcheckError::NotFound(reason) => {
                                tracing::warn!(
                                    message="healthcheck not found yet, waiting",
                                    service_name=%service_name,
                                    polling_interval=%polling_interval.as_millis(),
                                    reason=?reason,
                                );
                                tokio::time::sleep(polling_interval).await;
                            }
                        },
                    }
                }
            };

            tokio::time::timeout(timeout, client_fut.map_err(|e| e.into())).await?
        }
    }
}

pub mod server {
    use std::{
        future::Future,
        time::Duration,
    };

    use tokio::task::JoinHandle;
    use tonic::transport::NamedService;
    use tonic_health::proto::health_server::{
        Health,
        HealthServer,
    };

    use crate::protocol::healthcheck::{
        HealthcheckError,
        HealthcheckStatus,
    };

    pub async fn init_health_service<T, F, H>(
        healthcheck: H,
        healthcheck_polling_interval: Duration,
    ) -> (JoinHandle<()>, HealthServer<impl Health>)
    where
        T: NamedService,
        H: Fn() -> F + Send + Sync + 'static,
        F: Future<Output = Result<HealthcheckStatus, HealthcheckError>> + Send,
    {
        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

        // we configure our health reporter initially in the not_serving
        // state s.t. clients which are waiting for this service to start
        // can wait for the state change to the serving state
        health_reporter.set_not_serving::<T>().await;

        let healthcheck_handle = tokio::task::spawn(async move {
            // I initially tried to break this loop out into its own
            // function, but I ran into this issue:
            //
            // https://github.com/rust-lang/rust/issues/83701
            //
            // Unfortunately, the need to parametrize such a function by
            // self.healthcheck's type and the parameter S in
            // HealthReporter::set_serving<S>(..) makes this awkward, so I
            // just inlined the whole thing here.
            loop {
                match (healthcheck)().await {
                    Ok(status) => match status {
                        HealthcheckStatus::Serving => {
                            tracing::info!("healthcheck status \"serving\"");
                            health_reporter.set_serving::<T>().await
                        }
                        HealthcheckStatus::NotServing => {
                            tracing::warn!("healthcheck status \"not serving\"");
                            health_reporter.set_not_serving::<T>().await
                        }
                        HealthcheckStatus::Unknown => {
                            tracing::warn!("healthcheck status \"unknown\"");
                            health_reporter.set_not_serving::<T>().await
                        }
                    },
                    Err(e) => {
                        // healthcheck failed, so we'll set_not_serving()
                        tracing::error!(
                            message="healthcheck error",
                            error=?e
                        );
                        health_reporter.set_not_serving::<T>().await
                    }
                }

                tokio::time::sleep(healthcheck_polling_interval).await;
            }
        });

        (healthcheck_handle, health_service)
    }
}
