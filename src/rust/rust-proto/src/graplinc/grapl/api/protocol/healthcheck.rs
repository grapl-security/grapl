#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum HealthcheckError {
    #[error("not found {0}")]
    NotFound(String),

    #[error("healthcheck failed {0}")]
    HealthcheckFailed(String),

    #[error("failed to connect {0}")]
    ConnectionFailed(#[from] tonic::transport::Error),

    #[error("healthcheck timed out {0}")]
    TimeoutElapsed(#[from] tokio::time::error::Elapsed),
}

#[non_exhaustive]
#[derive(Debug)]
pub enum HealthcheckStatus {
    Serving,
    NotServing,
    Unknown,
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

    use crate::graplinc::grapl::api::protocol::healthcheck::{
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
