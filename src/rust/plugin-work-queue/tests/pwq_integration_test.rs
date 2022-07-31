#![cfg(feature = "integration_tests")]

use clap::Parser;
use rust_proto::{
    client_factory::{
        build_grpc_client_with_options,
        services::PluginWorkQueueClientConfig,
        BuildGrpcClientOptions,
    },
    graplinc::grapl::api::plugin_work_queue::v1beta1::{
        ExecutionJob,
        GetExecuteGeneratorRequest,
        PushExecuteGeneratorRequest,
    },
};

#[tokio::test]
async fn test_push_and_get_execute_generator() -> Result<(), Box<dyn std::error::Error>> {
    let mut pwq_client = build_grpc_client_with_options(
        PluginWorkQueueClientConfig::parse(),
        BuildGrpcClientOptions {
            perform_healthcheck: true,
            ..Default::default()
        },
    )
    .await?;

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
        .get_execute_generator(GetExecuteGeneratorRequest {
            plugin_id: plugin_id_2,
        })
        .await?;

    assert_eq!(
        retrieve_job_for_plugin_id_2.execution_job,
        Some(job_2.execution_job())
    );

    // Fetch for plugin_id 1
    let retrieve_job_for_plugin_id_1 = pwq_client
        .get_execute_generator(GetExecuteGeneratorRequest {
            plugin_id: plugin_id_1,
        })
        .await?;

    assert_eq!(
        retrieve_job_for_plugin_id_1.execution_job,
        Some(job_1.execution_job())
    );

    // Fetch for plugin_id 2 again, we should get Job 3
    let retrieve_job_for_plugin_id_2 = pwq_client
        .get_execute_generator(GetExecuteGeneratorRequest {
            plugin_id: plugin_id_2,
        })
        .await?;

    assert_eq!(
        retrieve_job_for_plugin_id_2.execution_job,
        Some(job_3.execution_job())
    );

    // Fetch one more time, we should be out of work
    let retrieve_job_for_plugin_id_2 = pwq_client
        .get_execute_generator(GetExecuteGeneratorRequest {
            plugin_id: plugin_id_2,
        })
        .await?;

    assert_eq!(retrieve_job_for_plugin_id_2.execution_job, None);

    Ok(())
}
