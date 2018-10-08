extern crate tower_grpc_build;

fn main() {
    // Build helloworld
    tower_grpc_build::Config::new()
        .enable_server(false)
        .enable_client(false)
        .build(&[
            "proto/ip_asset_mapping.proto",
        ], &["proto/"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}

