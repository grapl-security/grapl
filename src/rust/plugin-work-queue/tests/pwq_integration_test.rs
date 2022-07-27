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
async fn test_basic_crud_generator_pwq() -> Result<(), Box<dyn std::error::Error>> {
    let mut pwq_client = build_grpc_client_with_options(
        PluginWorkQueueClientConfig::parse(),
        BuildGrpcClientOptions {
            perform_healthcheck: true,
            ..Default::default()
        },
    )
    .await?;

    // Send 2 jobs to Plugin Work Queue
    let (plugin_id_1, plugin_id_2) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());

    let job_1 = PushExecuteGeneratorRequest {
        execution_job: ExecutionJob {
            data: "job 1".into(),
        },
        plugin_id: plugin_id_1,
    };

    let job_2 = PushExecuteGeneratorRequest {
        execution_job: ExecutionJob {
            data: "job 2".into(),
        },
        plugin_id: plugin_id_2,
    };

    for request in [&job_1, &job_2] {
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
        Some(job_2.execution_job)
    );

    let retrieve_job_for_plugin_id_1 = pwq_client
        .get_execute_generator(GetExecuteGeneratorRequest {
            plugin_id: plugin_id_1,
        })
        .await?;

    assert_eq!(
        retrieve_job_for_plugin_id_1.execution_job,
        Some(job_1.execution_job)
    );

    Ok(())
}
