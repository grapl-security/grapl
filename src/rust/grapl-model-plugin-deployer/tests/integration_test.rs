
#[cfg(integration)]
mod integration_tests {

    mod common;

    use common::ServiceContext;
    use test_context::futures;

    use grapl_model_plugin_deployer::client::GraplModelPluginDeployerRpcClient;
    use grapl_model_plugin_deployer::client::GraplModelPluginDeployerRequest;
    use grapl_model_plugin_deployer::client::Channel;
    use grapl_model_plugin_deployer::client::Timeout;

    use std::time::Duration;

    const MODEL_PLUGIN_SCHEMA: &str = r#"
type Process @grapl(identity_algorithm: "session") {
    process_name: String! @immutable,
    process_id: UInt! @pseudo_key,
    created_at: UInt! @create_time,
    last_seen_at: UInt! @last_seen_time,
    terminated_at: UInt! @terminate_time,
    binary_file: File! @edge(reverse: "executed_as_processes", reverse_relationship: "ToMany"),
    created_file: [File!] @edge(reverse: "created_by_process", reverse_relationship: "ToMany"),
}

type File @grapl(identity_algorithm: "session") {
    file_path: String! @pseudo_key,
    created_at: UInt! @create_time,
    last_seen_at: UInt! @last_seen_time,
    terminated_at: UInt! @terminate_time,
}

type SomePlugin @grapl(identity_algorithm: "static") {
    plugin_prop: String! @static_id,
}

type SomePluginExtendsProcess @grapl(extends: "Process") {
    process_to_plugin: Process! @edge(reverse: "get_the_plugin_node", reverse_relationship: "ToMany"),
}
"#;

    #[test_context::test_context(ServiceContext)]
    #[tokio::test]
    async fn smoketest(_ctx: &mut ServiceContext) -> Result<(), Box<dyn std::error::Error>> {
        let channel = Channel::from_static("http://[::1]:50051").connect().await?;

        let timeout_channel = Timeout::new(channel, Duration::from_millis(1000));

        let mut client = GraplModelPluginDeployerRpcClient::new(timeout_channel);

        let request = tonic::Request::new(GraplModelPluginDeployerRequest {
            model_plugin_schema: MODEL_PLUGIN_SCHEMA.to_owned(),
            schema_version: 1,
        });

        let _response = client.handle_request(request).await?;
        panic!("This test could use some work!");
    }

}
