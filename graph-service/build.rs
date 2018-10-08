extern crate tower_grpc_build;

fn main() {
    tower_grpc_build::Config::new()
        .enable_server(false)
        .enable_client(false)
        .build(&[
            "proto/subgraph_merge_event.proto",
        ], &["proto/"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}


