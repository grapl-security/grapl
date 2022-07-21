use std::time::Duration;

use rust_proto::protocol::{
    endpoint::Endpoint,
    healthcheck::client::HealthcheckClient,
    service_client::{
        ConnectError,
        Connectable,
        NamedService,
    },
};

use crate::grpc_client_config::{
    GenericGrpcClientConfig,
    GrpcClientConfig,
};

pub struct GetGrpcClientOptions {
    pub perform_healthcheck: bool,
    pub healthcheck_polling_interval: Duration,
}

impl Default for GetGrpcClientOptions {
    fn default() -> Self {
        GetGrpcClientOptions {
            perform_healthcheck: false,
            healthcheck_polling_interval: Duration::from_millis(500),
        }
    }
}

pub async fn get_grpc_client<C: GrpcClientConfig>(
    client_config: C,
) -> Result<C::Client, ConnectError> {
    get_grpc_client_with_options(client_config, Default::default()).await
}

pub async fn get_grpc_client_with_options<C: GrpcClientConfig>(
    client_config: C,
    options: GetGrpcClientOptions,
) -> Result<C::Client, ConnectError> {
    let GenericGrpcClientConfig { address } = client_config.into();

    let service_name = C::Client::SERVICE_NAME;

    if !address.starts_with("http") {
        panic!("Address should start with http, but found: '{address}'")
    }

    // TODO: Add a `rust-proto` wrapper around tonic Endpoint
    let endpoint = Endpoint::from_shared(address)?
        .timeout(Duration::from_secs(10))
        .concurrency_limit(30);

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
