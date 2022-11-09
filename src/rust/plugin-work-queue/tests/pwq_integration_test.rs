//#![cfg(feature = "integration_tests")]

use std::{
    assert_eq,
    time::Duration,
};

use figment::{
    providers::Env,
    Figment,
};
use rust_proto::graplinc::grapl::api::{
    client::{
        ClientError,
        Connect,
    },
    plugin_work_queue::v1beta1::{
        ExecutionJob,
        GetExecuteGeneratorRequest,
        PluginWorkQueueClient,
        PushExecuteAnalyzerRequest,
        PushExecuteGeneratorRequest,
        QueueDepthForAnalyzerRequest,
        QueueDepthForGeneratorRequest,
    },
    protocol::status::Code,
};
use test_context::{
    test_context,
    AsyncTestContext,
};

struct PwqTestContext {
    grpc_client: PluginWorkQueueClient,
}

#[async_trait::async_trait]
impl AsyncTestContext for PwqTestContext {
    async fn setup() -> Self {
        let client_config = Figment::new()
            .merge(Env::prefixed("PLUGIN_WORK_QUEUE_CLIENT_"))
            .extract()
            .expect("failed to extract p-w-q configuration");

        let pwq_client = PluginWorkQueueClient::connect(client_config)
            .await
            .expect("failed to connect to p-w-q");

        PwqTestContext {
            grpc_client: pwq_client,
        }
    }
}

#[test_context(PwqTestContext)]
#[test_log::test(tokio::test)]
async fn test_push_and_get_execute_generator(ctx: &mut PwqTestContext) -> eyre::Result<()> {
    // Send 2 jobs to Plugin Work Queue
    let tenant_id = uuid::Uuid::new_v4();
    let trace_id = uuid::Uuid::new_v4();
    let event_source_id = uuid::Uuid::new_v4();
    let (plugin_id_1, plugin_id_2) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());

    let job_1 = PushExecuteGeneratorRequest::new(
        ExecutionJob::new(
            "for plugin_id_1".into(),
            tenant_id,
            trace_id,
            event_source_id,
        ),
        plugin_id_1,
    );

    let job_2 = PushExecuteGeneratorRequest::new(
        ExecutionJob::new(
            "for plugin_id_2".into(),
            tenant_id,
            trace_id,
            event_source_id,
        ),
        plugin_id_2,
    );

    let job_3 = PushExecuteGeneratorRequest::new(
        ExecutionJob::new(
            "also for plugin_id_2".into(),
            tenant_id,
            trace_id,
            event_source_id,
        ),
        plugin_id_2,
    );

    for request in [&job_1, &job_2, &job_3] {
        ctx.grpc_client
            .push_execute_generator(request.clone())
            .await?;
    }

    // Now retrieve the jobs and assert we got the right ones.

    // I retrieve job 2 first on purpose; it ellicits the unexpected behavior where
    // GetExecuteGenerator doesn't pay attention to the plugin_id *at all*.

    let retrieve_job_for_plugin_id_2 = ctx
        .grpc_client
        .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id_2))
        .await?;

    assert_eq!(
        retrieve_job_for_plugin_id_2.execution_job(),
        Some(job_2.execution_job())
    );

    // Fetch for plugin_id 1
    let retrieve_job_for_plugin_id_1 = ctx
        .grpc_client
        .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id_1))
        .await?;

    assert_eq!(
        retrieve_job_for_plugin_id_1.execution_job(),
        Some(job_1.execution_job())
    );

    // Fetch for plugin_id 2 again, we should get Job 3
    let retrieve_job_3_for_plugin_id_2 = ctx
        .grpc_client
        .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id_2))
        .await?;

    assert_eq!(
        retrieve_job_3_for_plugin_id_2.execution_job(),
        Some(job_3.execution_job())
    );

    // Fetch one more time, we should be out of work
    let retrieve_job_none_for_plugin_id_2 = ctx
        .grpc_client
        .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id_2))
        .await?;

    assert_eq!(retrieve_job_none_for_plugin_id_2.execution_job(), None);

    Ok(())
}

#[test_context(PwqTestContext)]
#[test_log::test(tokio::test)]
async fn test_message_available_after_failure(ctx: &mut PwqTestContext) -> eyre::Result<()> {
    // Send a job to Plugin Work Queue
    let tenant_id = uuid::Uuid::new_v4();
    let trace_id = uuid::Uuid::new_v4();
    let event_source_id = uuid::Uuid::new_v4();
    let plugin_id = uuid::Uuid::new_v4();

    let job = PushExecuteGeneratorRequest::new(
        ExecutionJob::new("the only job".into(), tenant_id, trace_id, event_source_id),
        plugin_id,
    );

    ctx.grpc_client.push_execute_generator(job.clone()).await?;

    let retrieve_job = move || {
        let mut pwq_client = ctx.grpc_client.clone();
        async move {
            pwq_client
                .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id))
                .await
                .map_err(|e| eyre::eyre!(e))
        }
    };

    // Get the job
    let retrieved_job = retrieve_job().await?.execution_job();
    eyre::ensure!(
        retrieved_job == Some(job.clone().execution_job()),
        "Expected job equality: {:?}, {:?}",
        retrieved_job,
        job
    );

    // If we get the job again, it should be None this time.
    let retrieved_job = retrieve_job().await?.execution_job();
    eyre::ensure!(
        retrieved_job.is_none(),
        "Expected None job: {:?}",
        retrieved_job,
    );

    // If we haven't acknowledged it for 10 seconds, it becomes visible again
    tokio::time::sleep(Duration::from_millis(10_500)).await;

    let retrieved_job = retrieve_job().await?.execution_job();
    eyre::ensure!(
        retrieved_job == Some(job.clone().execution_job()),
        "Expected job equality: {:?}, {:?}",
        retrieved_job,
        job
    );

    Ok(())
}

#[test_context(PwqTestContext)]
#[test_log::test(tokio::test)]
async fn test_queue_depth_for_generator(ctx: &mut PwqTestContext) -> eyre::Result<()> {
    // Send 2 jobs to Plugin Work Queue
    let tenant_id = uuid::Uuid::new_v4();
    let (event_source_id_1, event_source_id_2) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
    let (generator_id_1, generator_id_2) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());

    let job_1 = PushExecuteGeneratorRequest::new(
        ExecutionJob::new(
            "for generator 1".into(),
            tenant_id,
            uuid::Uuid::new_v4(),
            event_source_id_1,
        ),
        generator_id_1,
    );

    let job_2 = PushExecuteGeneratorRequest::new(
        ExecutionJob::new(
            "for generator 2".into(),
            tenant_id,
            uuid::Uuid::new_v4(),
            event_source_id_2,
        ),
        generator_id_2,
    );

    let job_3 = PushExecuteGeneratorRequest::new(
        ExecutionJob::new(
            "also for generator 2".into(),
            tenant_id,
            uuid::Uuid::new_v4(),
            event_source_id_2,
        ),
        generator_id_2,
    );

    for request in [&job_1, &job_2, &job_3] {
        ctx.grpc_client
            .push_execute_generator(request.clone())
            .await?;
    }

    let response_1 = ctx
        .grpc_client
        .queue_depth_for_generator(QueueDepthForGeneratorRequest::new(generator_id_1))
        .await?;

    assert_eq!(response_1.queue_depth(), 1);
    assert_eq!(response_1.event_source_id(), event_source_id_1);

    let response_2 = ctx
        .grpc_client
        .queue_depth_for_generator(QueueDepthForGeneratorRequest::new(generator_id_2))
        .await?;

    assert_eq!(response_2.queue_depth(), 2);
    assert_eq!(response_2.event_source_id(), event_source_id_2);

    Ok(())
}

#[test_context(PwqTestContext)]
#[test_log::test(tokio::test)]
async fn test_queue_depth_for_generator_not_found(ctx: &mut PwqTestContext) {
    match ctx
        .grpc_client
        .queue_depth_for_generator(QueueDepthForGeneratorRequest::new(uuid::Uuid::new_v4()))
        .await
    {
        Ok(_) => panic!("expected error response"),
        Err(e) => {
            if let ClientError::Status(status) = e {
                let status_code = status.code();
                if let Code::NotFound = status_code {
                    // 👍 great success 👍
                } else {
                    panic!("expected Code::NotFound, encountered {}", status_code);
                }
            } else {
                panic!("expected ClientError::Status, encountered {}", e);
            }
        }
    }
}

#[test_context(PwqTestContext)]
#[test_log::test(tokio::test)]
async fn test_queue_depth_for_analyzer(ctx: &mut PwqTestContext) -> eyre::Result<()> {
    // Send 2 jobs to Plugin Work Queue
    let tenant_id = uuid::Uuid::new_v4();
    let (event_source_id_1, event_source_id_2) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
    let (analyzer_id_1, analyzer_id_2) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());

    let job_1 = PushExecuteAnalyzerRequest::new(
        ExecutionJob::new(
            "for analyzer 1".into(),
            tenant_id,
            uuid::Uuid::new_v4(),
            event_source_id_1,
        ),
        analyzer_id_1,
    );

    let job_2 = PushExecuteAnalyzerRequest::new(
        ExecutionJob::new(
            "for analyzer 2".into(),
            tenant_id,
            uuid::Uuid::new_v4(),
            event_source_id_1,
        ),
        analyzer_id_2,
    );

    let job_3 = PushExecuteAnalyzerRequest::new(
        ExecutionJob::new(
            "also for analyzer 2".into(),
            tenant_id,
            uuid::Uuid::new_v4(),
            event_source_id_2,
        ),
        analyzer_id_2,
    );

    let job_4 = PushExecuteAnalyzerRequest::new(
        ExecutionJob::new(
            "again for analyzer 2".into(),
            tenant_id,
            uuid::Uuid::new_v4(),
            event_source_id_1,
        ),
        analyzer_id_2,
    );

    for request in [&job_1, &job_2, &job_3, &job_4] {
        ctx.grpc_client
            .push_execute_analyzer(request.clone())
            .await?;
    }

    let response_1 = ctx
        .grpc_client
        .queue_depth_for_analyzer(QueueDepthForAnalyzerRequest::new(analyzer_id_1))
        .await?;

    assert_eq!(response_1.queue_depth(), 1);
    assert_eq!(response_1.dominant_event_source_id(), event_source_id_1);

    let response_2 = ctx
        .grpc_client
        .queue_depth_for_analyzer(QueueDepthForAnalyzerRequest::new(analyzer_id_2))
        .await?;

    assert_eq!(response_2.queue_depth(), 3);
    assert_eq!(response_2.dominant_event_source_id(), event_source_id_1);

    Ok(())
}

#[test_context(PwqTestContext)]
#[test_log::test(tokio::test)]
async fn test_queue_depth_for_analyzer_not_found(ctx: &mut PwqTestContext) {
    match ctx
        .grpc_client
        .queue_depth_for_analyzer(QueueDepthForAnalyzerRequest::new(uuid::Uuid::new_v4()))
        .await
    {
        Ok(_) => panic!("expected error response"),
        Err(e) => {
            if let ClientError::Status(status) = e {
                let status_code = status.code();
                if let Code::NotFound = status_code {
                    // 👍 great success 👍
                } else {
                    panic!("expected Code::NotFound, encountered {}", status_code);
                }
            } else {
                panic!("expected ClientError::Status, encountered {}", e);
            }
        }
    }
}
