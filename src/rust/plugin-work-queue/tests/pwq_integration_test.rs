#![cfg(feature = "integration_tests")]

use std::time::Duration;

use figment::{
    providers::Env,
    Figment,
};
use rust_proto::graplinc::grapl::api::{
    client::Connect,
    plugin_work_queue::v1beta1::{
        ExecutionJob,
        GetExecuteGeneratorRequest,
        PluginWorkQueueClient,
        PushExecuteGeneratorRequest,
    },
};

#[tokio::test]
async fn test_push_and_get_execute_generator() -> eyre::Result<()> {
    let client_config = Figment::new()
        .merge(Env::prefixed("PLUGIN_WORK_QUEUE_CLIENT_"))
        .extract()?;
    let mut pwq_client = PluginWorkQueueClient::connect(client_config).await?;

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
        pwq_client.push_execute_generator(request.clone()).await?;
    }

    // Now retrieve the jobs and assert we got the right ones.

    // I retrieve job 2 first on purpose; it ellicits the unexpected behavior where
    // GetExecuteGenerator doesn't pay attention to the plugin_id *at all*.

    let retrieve_job_for_plugin_id_2 = pwq_client
        .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id_2))
        .await?;

    assert_eq!(
        retrieve_job_for_plugin_id_2.execution_job(),
        Some(job_2.execution_job())
    );

    // Fetch for plugin_id 1
    let retrieve_job_for_plugin_id_1 = pwq_client
        .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id_1))
        .await?;

    assert_eq!(
        retrieve_job_for_plugin_id_1.execution_job(),
        Some(job_1.execution_job())
    );

    // Fetch for plugin_id 2 again, we should get Job 3
    let retrieve_job_3_for_plugin_id_2 = pwq_client
        .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id_2))
        .await?;

    assert_eq!(
        retrieve_job_3_for_plugin_id_2.execution_job(),
        Some(job_3.execution_job())
    );

    // Fetch one more time, we should be out of work
    let retrieve_job_none_for_plugin_id_2 = pwq_client
        .get_execute_generator(GetExecuteGeneratorRequest::new(plugin_id_2))
        .await?;

    assert_eq!(retrieve_job_none_for_plugin_id_2.execution_job(), None);

    Ok(())
}

#[tokio::test]
async fn test_message_available_after_failure() -> eyre::Result<()> {
    let client_config = Figment::new()
        .merge(Env::prefixed("PLUGIN_WORK_QUEUE_CLIENT_"))
        .extract()?;
    let mut pwq_client = PluginWorkQueueClient::connect(client_config).await?;

    // Send a job to Plugin Work Queue
    let tenant_id = uuid::Uuid::new_v4();
    let trace_id = uuid::Uuid::new_v4();
    let event_source_id = uuid::Uuid::new_v4();
    let plugin_id = uuid::Uuid::new_v4();

    let job = PushExecuteGeneratorRequest::new(
        ExecutionJob::new("the only job".into(), tenant_id, trace_id, event_source_id),
        plugin_id,
    );

    pwq_client.push_execute_generator(job.clone()).await?;

    let retrieve_job = move || {
        let mut pwq_client = pwq_client.clone();
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
