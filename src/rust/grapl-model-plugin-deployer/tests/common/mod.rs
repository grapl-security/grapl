use std::time::Duration;

use grapl_model_plugin_deployer_lib::{
    server::{
        GraplModelPluginDeployer,
        GraplModelPluginDeployerRpcServer,
    },
};
use test_context::AsyncTestContext;
use tonic_health::proto::{
    health_check_response::ServingStatus,
    health_client::HealthClient,
    HealthCheckRequest,
};
use tonic::transport::NamedService;

pub struct ServiceContext {}

#[async_trait::async_trait]
impl AsyncTestContext for ServiceContext {
    async fn setup() -> Self {
        let _subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .init();

        tokio::task::spawn(async move {
            grapl_model_plugin_deployer_lib::server::exec_service()
                .await
                .expect("Failed to execute service");
        });
        until_health().await.expect("Service was never healthy");
        Self {}
    }

    async fn teardown(self) {}
}

async fn until_health() -> Result<(), Box<dyn std::error::Error>> {
    for i in 0.. {
        match _until_health().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if i == 5 {
                    tracing::error!(error=?e, times=i, message="Health Check failed");
                    return Err(e);
                }
                tracing::debug!(error=?e, times=i, message="Health Check failed");
            }
        }
        tokio::time::sleep(Duration::from_millis(i * 10)).await;
    }
    unreachable!()
}

async fn _until_health() -> Result<(), Box<dyn std::error::Error>> {
    let channel = tonic::transport::Channel::from_static("http://[::1]:50051").connect().await?;

    let timeout_channel = tower::timeout::Timeout::new(channel, Duration::from_millis(1000));

    let mut client = HealthClient::new(timeout_channel);

    let request = HealthCheckRequest {
        service: GraplModelPluginDeployerRpcServer::<GraplModelPluginDeployer>::NAME.to_string(),
    };
    let response = client.check(request).await?;
    let response = response.into_inner();
    match response.status() {
        ServingStatus::Serving => Ok(()),
        other => Err(format!("Not serving: {:?}", other).into()),
    }
}
