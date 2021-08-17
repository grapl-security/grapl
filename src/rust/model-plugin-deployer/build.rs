fn main() {
    let config = tonic_build::configure();

    config
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../proto/graplinc/grapl/api/model_plugin_deployer/v1/model_plugin_deployer.proto",
                "../../proto/graplinc/grapl/api/model_plugin_deployer/v1/model_plugin_deployer_health.proto",
            ],
            &["../../proto/"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
