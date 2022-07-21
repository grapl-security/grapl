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

use crate::grpc_client_config::GrpcClientConfig;

pub async fn get_grpc_client<C: GrpcClientConfig>(
    client_config: C,
) -> Result<C::Client, ConnectError> {
    let address = client_config.address();
    let service_name = C::Client::SERVICE_NAME;

    /*
    let address = address.to_string();
    if ! address.starts_with("http") {
        panic!("Address should start with http, but found: '{address}'")
    }
    */

    let endpoint = Endpoint::from_shared(address)?
        .timeout(Duration::from_secs(10))
        .concurrency_limit(30);

    HealthcheckClient::wait_until_healthy(
        endpoint.clone(),
        service_name,
        Duration::from_millis(10000),
        Duration::from_millis(client_config.healthcheck_polling_interval_ms()),
    )
    .await?;

    C::Client::connect(endpoint).await
}
