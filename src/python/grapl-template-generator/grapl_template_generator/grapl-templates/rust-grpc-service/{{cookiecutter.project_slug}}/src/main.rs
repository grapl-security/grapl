use std::time::Duration;

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _subscriber = ::tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let builder = PrometheusBuilder::new();
    builder
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(Duration::from_secs(10)),
        )
        .install()
        .expect("failed to install Prometheus recorder");

    {{cookiecutter.snake_project_name}}::server::exec_service().await?;
    Ok(())
}
