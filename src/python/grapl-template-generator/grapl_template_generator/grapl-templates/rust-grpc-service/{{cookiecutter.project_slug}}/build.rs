fn main() {
    let config = tonic_build::configure();

    config
        .build_server(true)
        .build_client(true)
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
