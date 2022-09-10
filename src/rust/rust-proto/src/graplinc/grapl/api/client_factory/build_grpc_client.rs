use std::time::Duration;

use crate::graplinc::grapl::api::{
    client_factory::grpc_client_config::{
        GenericGrpcClientConfig,
        GrpcClientConfig,
    },
    protocol::{
        endpoint::Endpoint,
        healthcheck::client::HealthcheckClient,
        service_client::{
            ConnectError,
            Connectable,
        },
    },
};

#[derive(Clone)]
struct BuildGrpcClientOptions {
    pub perform_healthcheck: bool,
    pub healthcheck_polling_interval: Duration,
}

impl Default for BuildGrpcClientOptions {
    fn default() -> Self {
        BuildGrpcClientOptions {
            perform_healthcheck: false,
            healthcheck_polling_interval: Duration::from_millis(500),
        }
    }
}

pub async fn build_grpc_client<C: GrpcClientConfig>(
    client_config: C,
) -> Result<C::Client, ConnectError> {
    build_grpc_client_with_options(client_config, Default::default()).await
}

async fn build_grpc_client_with_options<C: GrpcClientConfig>(
    client_config: C,
    options: BuildGrpcClientOptions,
) -> Result<C::Client, ConnectError> {
    let GenericGrpcClientConfig { address } = client_config.into();

    let service_name = C::Client::SERVICE_NAME;

    if !address.starts_with("http") {
        panic!("Address should start with http, but found: '{address}'")
    }

    // TODO: Add a `rust-proto` wrapper around tonic Endpoint
    let endpoint = Endpoint::from_shared(address)?
        .timeout(Duration::from_secs(10))
        .concurrency_limit(300);

    if options.perform_healthcheck {
        HealthcheckClient::wait_until_healthy(
            endpoint.clone(),
            service_name,
            Duration::from_millis(10000),
            options.healthcheck_polling_interval,
        )
        .await?;
    }

    C::Client::connect(endpoint).await
}
