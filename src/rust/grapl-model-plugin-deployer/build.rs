fn main() {
    let config = tonic_build::configure();

    config
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "proto/grapl_model_plugin_deployer.proto",
            ],
            &["proto/"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
