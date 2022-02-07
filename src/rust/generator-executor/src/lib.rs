use async_stream::stream;
use futures_util::pin_mut;
use generator_sdk::{
    client::{
        CacheBuilder,
        GeneratorClient,
        GeneratorClientError,
        TokioAsyncResolver,
    },
    ClientConfig as GeneratorClientConfig,
};
use grapl_utils::future_ext::GraplFutureExt;
use plugin_work_queue::client::PluginWorkQueueServiceClient;
use rust_proto::plugin_work_queue::{
    AcknowledgeGeneratorRequest,
    ExecutionJob,
    GetExecuteGeneratorRequest,
    GetExecuteGeneratorResponse,
};
use structopt::StructOpt;
use tokio::time::{
    sleep,
    Duration,
};
use tokio_stream::{
    Stream,
    StreamExt,
};

#[derive(StructOpt)]
pub struct GeneratorExecutorConfig {
    #[structopt(env)]
    plugin_work_queue_address: String,
    // Path to the certificate to be used when validating the generator plugin TLS sessions
    #[structopt(env)]
    generator_certificate_path: std::path::PathBuf,
    #[structopt(flatten)]
    generator_client_config: GeneratorClientConfig,
}

fn make_job_stream(
    mut work_queue_client: PluginWorkQueueServiceClient<tonic::transport::Channel>,
) -> impl Stream<Item = (ExecutionJob, i64)> {
    stream! {
        loop {
            let message = match work_queue_client.get_execute_generator(GetExecuteGeneratorRequest {}).await {
                Ok(message) => message,
                Err(e) => {
                    tracing::error!(message="Failed to retrieve generator job", error=?e);
                    // check what kind of error it is and retry it accordingly
                    sleep(Duration::from_secs(1)).await;
                    continue
                }
            };
            match message {
                GetExecuteGeneratorResponse { execution_job: Some(job), request_id } => yield (job, request_id),
                _ => {
                    tracing::debug!(message="No available jobs");
                    sleep(Duration::from_secs(1)).await;
                }
            };

        }
    }
}

// Attempts to ack the message. On failure, will retry in a background task
async fn try_ack(
    work_queue_client: &mut PluginWorkQueueServiceClient<tonic::transport::Channel>,
    request_id: i64,
    success: bool,
) {
    if let Err(e) = work_queue_client
        .acknowledge_generator(AcknowledgeGeneratorRequest {
            request_id,
            success,
        })
        .timeout(Duration::from_secs(2))
        .await
    {
        let mut work_queue_client = work_queue_client.clone();
        tokio::spawn(async move {
            tracing::error!(
                message="Failed to acknowledge message after 2 seconds",
                error=?e,
            );
            if let Err(e) = work_queue_client
                .acknowledge_generator(AcknowledgeGeneratorRequest {
                    request_id,
                    success,
                })
                .timeout(Duration::from_secs(10))
                .await
            {
                tracing::error!(
                    message="Failed to acknowledge message after 10 seconds",
                    error=?e,
                )
            }
        });
    };
}

pub async fn process_message_loop(
    mut work_queue_client: PluginWorkQueueServiceClient<tonic::transport::Channel>,
    mut generator_client: GeneratorClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let jobs = make_job_stream(work_queue_client.clone());
    pin_mut!(jobs);

    while let Some((job, request_id)) = jobs.next().await {
        let ExecutionJob {
            tenant_id: _,
            plugin_id,
            data,
        } = job;

        // based on the error, either retry later or ack as a failure
        let response = generator_client
            .run_generator(data, plugin_id.to_string())
            .await;

        match response {
            Ok(_) => {
                try_ack(&mut work_queue_client, request_id, true).await;
            }
            Err(ref e) => {
                if !should_retry(e) {
                    try_ack(&mut work_queue_client, request_id, false).await;
                }
            }
        }
    }

    Ok(())
}

// Unless there's a reason to believe this error is persistent, we should always retry
fn should_retry(client_error: &GeneratorClientError) -> bool {
    match client_error {
        GeneratorClientError::ConnectError(_) => true,
        GeneratorClientError::EmptyResolution { .. } => true,
        GeneratorClientError::ResolveError(_) => true,
        GeneratorClientError::InvalidUri(_) => false,
        GeneratorClientError::Status(_) => true,
        GeneratorClientError::ProtocolError(_) => true,
        GeneratorClientError::ProtoError(_) => false,
    }
}

pub async fn execute_service(
    config: GeneratorExecutorConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let cert = std::fs::read(config.generator_certificate_path)?;
    let cert = tonic::transport::Certificate::from_pem(cert);

    let work_queue =
        PluginWorkQueueServiceClient::connect(config.plugin_work_queue_address).await?;
    let generator_client = GeneratorClient::new(
        CacheBuilder::from(config.generator_client_config.client_cache_config).build(),
        cert, // load certificate
        TokioAsyncResolver::from(config.generator_client_config.client_dns_config),
    );
    process_message_loop(work_queue, generator_client).await?;

    Ok(())
}
